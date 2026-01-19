# SIMD-Accelerated Bencodex Parsing

This document describes the SIMD (Single Instruction, Multiple Data) accelerated parsing implementation in bencodex-rs.

## Overview

The SIMD decoder uses a **two-stage parsing strategy** inspired by [simdjson](https://github.com/simdjson/simdjson):

1. **Stage 1 (Structural Scanning)**: Uses SIMD instructions to scan the entire input and identify positions of all structural characters
2. **Stage 2 (Value Extraction)**: Uses the structural index to parse values efficiently by jumping directly to relevant positions

## Why SIMD for Bencodex?

Bencodex has several properties that make it well-suited for SIMD optimization:

| Property | Benefit |
|----------|---------|
| **No escape sequences** | Unlike JSON, strings don't need escape processing |
| **Length-prefixed strings** | Clear boundaries without scanning for delimiters |
| **Integer-only numbers** | No floating-point parsing complexity |
| **Single-byte type markers** | Easy SIMD comparison (`n`, `t`, `f`, `i`, `l`, `d`, `u`) |
| **Canonical representation** | Predictable parsing paths |

## Supported Architectures

| Architecture | Instruction Set | Vector Width | Runtime Detection |
|--------------|-----------------|--------------|-------------------|
| x86_64 | AVX2 | 256-bit (32 bytes) | Yes |
| x86_64 | SSE4.2 | 128-bit (16 bytes) | Yes |
| AArch64 | NEON | 128-bit (16 bytes) | Always available |

The decoder automatically selects the best available instruction set at runtime.

## Architecture

### Module Structure

```
src/codec/simd/
├── mod.rs              # Public API, runtime dispatcher
├── arch/
│   ├── mod.rs          # SimdBackend trait definition
│   ├── x86_64.rs       # SSE4.2/AVX2 implementations
│   ├── aarch64.rs      # NEON implementation
│   └── fallback.rs     # Scalar fallback
├── structural.rs       # StructuralIndex type
├── stage1.rs           # Structural character scanning
├── stage2.rs           # Value extraction parser
└── number.rs           # Number parsing utilities
```

### SimdBackend Trait

The `SimdBackend` trait abstracts over different SIMD instruction sets:

```rust
pub trait SimdBackend {
    const LANE_WIDTH: usize;  // 16 (SSE/NEON) or 32 (AVX2)
    type Vector: Copy;

    unsafe fn load_unaligned(ptr: *const u8) -> Self::Vector;
    unsafe fn cmpeq_epi8(a: Self::Vector, b: u8) -> Self::Vector;
    unsafe fn or(a: Self::Vector, b: Self::Vector) -> Self::Vector;
    unsafe fn movemask_epi8(a: Self::Vector) -> u32;
}
```

### Stage 1: Structural Index Building

Stage 1 scans the input using SIMD to find all **structural characters**:

- Type markers: `n`, `t`, `f`, `i`, `l`, `d`, `u`
- Delimiters: `:`, `e`
- Digits: `0-9`

The algorithm:
1. Load 16/32 bytes into a SIMD register
2. Compare against each structural character using `cmpeq`
3. Combine masks with OR operations
4. Extract positions from the bitmask using `trailing_zeros`
5. Store positions in a `Vec<u32>`

```rust
// Pseudocode for structural scanning
for chunk in input.chunks(LANE_WIDTH) {
    let vec = load_unaligned(chunk);

    let mask_n = cmpeq(vec, b'n');
    let mask_t = cmpeq(vec, b't');
    // ... more comparisons

    let combined = or(mask_n, or(mask_t, ...));
    let bitmask = movemask(combined);

    while bitmask != 0 {
        let pos = bitmask.trailing_zeros();
        indices.push(offset + pos);
        bitmask &= bitmask - 1;  // Clear lowest bit
    }
}
```

### Stage 2: Value Extraction

Stage 2 uses the structural index to parse values:

```rust
pub struct SimdParser<'a> {
    input: &'a [u8],
    structural: &'a StructuralIndex,
    pos: usize,
    struct_idx: usize,  // Cursor into structural index
}
```

Key optimization: Instead of scanning byte-by-byte for delimiters, the parser looks up positions in the structural index:

```rust
// Find next ':' using structural index (O(k) where k = index entries)
fn find_next_structural(&mut self, byte: u8) -> Option<usize> {
    while self.struct_idx < self.structural.len() {
        let pos = self.structural.get(self.struct_idx)? as usize;
        self.struct_idx += 1;
        if pos >= self.pos && self.input[pos] == byte {
            return Some(pos);
        }
    }
    None
}
```

## Usage

### Enable the Feature

```toml
[dependencies]
bencodex = { version = "0.6", features = ["simd"] }
```

### Using the Trait Method

```rust
use bencodex::Decode;

let data = b"li1ei2ei3ee".to_vec();
let value = data.decode_simd().unwrap();
```

### Using the Function Directly

```rust
use bencodex::simd::decode_simd;

let value = decode_simd(b"d3:keyu5:valueee").unwrap();
```

## Performance Characteristics

### When SIMD Helps

- **Large inputs**: Amortizes Stage 1 overhead
- **Many structural characters**: More positions to skip
- **Long strings**: Skip data portions without scanning
- **Deep nesting**: Many delimiters to find quickly

### When SIMD May Not Help

- **Very small inputs** (< 32 bytes): Stage 1 overhead dominates
- **Simple values**: Single integers or short strings
- **Already memory-bound**: SIMD won't help if waiting for memory

### Theoretical Throughput

| Implementation | Expected Throughput |
|----------------|---------------------|
| Scalar | 70-80 MiB/s |
| SIMD Stage 1 only | 3-8 GB/s |
| SIMD Full (Stage 1 + 2) | 100-200 MiB/s |

## Implementation Notes

### NEON movemask Emulation

ARM NEON doesn't have a direct `movemask` equivalent. We emulate it:

```rust
unsafe fn neon_movemask(v: uint8x16_t) -> u32 {
    // Shift right by 7 to get MSB in bit 0
    let shifted = vshrq_n_u8::<7>(v);

    // Multiply by bit position weights [1,2,4,8,16,32,64,128,...]
    let bit_pos = vld1q_u8([1,2,4,8,16,32,64,128,1,2,4,8,16,32,64,128].as_ptr());
    let masked = vmulq_u8(shifted, bit_pos);

    // Sum each 8-byte half
    let low_sum = vaddv_u8(vget_low_u8(masked));
    let high_sum = vaddv_u8(vget_high_u8(masked));

    low_sum as u32 | ((high_sum as u32) << 8)
}
```

### Structural Index Contents

The structural index contains positions of **all potential** structural characters, including those inside string data. The Stage 2 parser must:

1. Track current parse position
2. Skip index entries that are inside string data
3. Validate that found positions contain expected bytes

## Future Optimizations

1. **Typed structural index**: Store character type alongside position to avoid re-checking bytes

2. **Small input fast path**: Skip SIMD for inputs < 32 bytes

3. **SIMD number parsing**: Use SIMD for multi-digit integer conversion

4. **On-demand search**: For specific patterns, search on-demand instead of pre-building full index

5. **Parallel Stage 1**: Split large inputs across threads for Stage 1 scanning

## References

- [simdjson: Parsing Gigabytes of JSON per Second](https://arxiv.org/abs/1902.08318)
- [Bencodex Specification](https://github.com/planetarium/bencodex)
- [Intel Intrinsics Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/)
- [ARM NEON Intrinsics Reference](https://developer.arm.com/architectures/instruction-sets/intrinsics/)
