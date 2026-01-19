# bencodex-rs

[![build](https://github.com/bencodex/bencodex-rs/actions/workflows/build.yaml/badge.svg)](https://github.com/bencodex/bencodex-rs/actions/workflows/build.yaml) [![codecov](https://codecov.io/gh/moreal/bencodex-rs/graph/badge.svg?token=H0FWUZ2ZF2)](https://codecov.io/gh/moreal/bencodex-rs) [![Docs-rs](https://docs.rs/bencodex-rs/badge.svg)](https://docs.rs/bencodex-rs/latest/)

The [Rust] implementation of [Bencodex].

- **Correctness** - Implement Bencodex spec and passed tests with its testsuites.
- **[Bencodex JSON]** - Support encoding Bencodex to JSON and decoding JSON to Bencodex.
- **Feature flags** - Support `std`, `alloc`, `json`, `json-cli` feature flags to minimize binary size in use.
- **`no_std` support** - Can be used in embedded environments with `alloc`.

[Rust]: https://rust-lang.org/
[Bencodex]: https://bencodex.org/

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Enables standard library support |
| `alloc` | Yes (via `std`) | Enables `alloc` crate for `no_std` environments |
| `json` | No | Enables Bencodex JSON encoding/decoding |
| `json-cli` | No | Enables CLI tool for JSON conversion |
| `simd` | No | Enables SIMD-accelerated decoding |

## SIMD Decoding

The `simd` feature provides SIMD-accelerated decoding for improved performance on large data. This feature requires `std` and is not available in `no_std` environments.

**Supported architectures:**
- x86_64: SSE4.2 and AVX2 (runtime detection)
- AArch64: NEON

```toml
[dependencies]
bencodex-rs = { version = "<VERSION>", features = ["simd"] }
```

```rust
use bencodex::{Decode, BencodexValue};

// Using trait method
let data = b"i42e".to_vec();
let value = data.decode_simd().unwrap();

// Or using the function directly
use bencodex::simd::decode_simd;
let value = decode_simd(b"li1ei2ei3ee").unwrap();
```

## `no_std` Support

This crate supports `no_std` environments with the `alloc` crate. To use in a `no_std` environment:

```toml
[dependencies]
bencodex-rs = { version = "<VERSION>", default-features = false, features = ["alloc"] }
```

**Notes:**
- `no_std` support requires Rust 1.81+ (for `core::error::Error` stabilization).
- The `json` feature requires `std` and is not available in `no_std` environments.

## Bencodex JSON feature

bencodex-rs implements [Bencodex JSON] feature, encoding and decoding both.

To use Bencodex JSON feature, you should enable `json` feature.

```toml
bencodex-rs = { version = "<VERSION>", features = ["json"] }
```

[Bencodex JSON]: https://github.com/planetarium/bencodex/blob/main/JSON.md

### Encoding to JSON

To encode from Bencodex to JSON, you can use `to_json` function.

```rust
use bencodex::{ BencodexValue, json::to_json };

let json = to_json(&BencodexValue::Null).expect("Failed to encode JSON.");
println!("{}", json);
```

There are two ways to encode `BencodexValue::Binary` type, `Hex` and `Base64`. You can choose one way with `bencodex::json::BinaryEncoding`. And you can pass it with `bencodex::json::JsonEncodeOptions` to `bencodex::json::to_json_with_options`.

```rust
use bencodex::BencodexValue;
use bencodex::json::{ BinaryEncoding, JsonEncodeOptions, to_json_with_options };

let json = to_json_with_options(&BencodexValue::Null, JsonEncodeOptions {
  binary_encoding: BinaryEncoding::Base64,
}).expect("Failed to encode JSON.");
println!("{}", json);
```

### Decoding from JSON

To decode from JSON to Bencodex, you can use `from_json_string` and `from_json` function.

```rust
// from_json_string
use bencodex::{ BencodexValue, json::from_json_string };

let result = from_json_string("null");
assert!(result.is_ok());
assert_eq!(result.unwrap(), BencodexValue::Null);
```

```rust
// from_json
use serde_json::from_str;
use bencodex::{ BencodexValue, json::from_json };

let json = from_str("null").unwrap();
let result = from_json(&json);
assert!(result.is_ok());
assert_eq!(result.unwrap(), BencodexValue::Null);
```

### CLI Tool


Also, it provides a CLI tool to encode from Bencodex to JSON and to decode from JSON to Bencodex. You can install it with `json-cli` feature like the below line:

```bash
cargo install bencodex-rs --features json-cli
```

You can use like the below:

```bash
# encode
$ echo -n 'n' | bencodex
null
$ echo -n 'i123e' | bencodex
"123"
$ echo -n '1:\x12' | bencodex
"0x12"
$ echo -n '1:\x12' | bencodex --base64
"b64:Eg=="

# decode
$ echo -n '"123"' | bencodex -d
123
$ echo -n 'null' | bencodex -d
n
```
