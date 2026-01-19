//! x86_64 SIMD backends (SSE4.2 and AVX2)

#![allow(unused_unsafe)]

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::SimdBackend;

/// SSE4.2 backend (128-bit vectors)
pub struct Sse42;

impl SimdBackend for Sse42 {
    const LANE_WIDTH: usize = 16;
    type Vector = __m128i;

    #[inline]
    #[target_feature(enable = "sse4.2")]
    unsafe fn load_unaligned(ptr: *const u8) -> Self::Vector {
        unsafe { _mm_loadu_si128(ptr as *const __m128i) }
    }

    #[inline]
    #[target_feature(enable = "sse4.2")]
    unsafe fn cmpeq_epi8(a: Self::Vector, b: u8) -> Self::Vector {
        unsafe {
            let broadcast = _mm_set1_epi8(b as i8);
            _mm_cmpeq_epi8(a, broadcast)
        }
    }

    #[inline]
    #[target_feature(enable = "sse4.2")]
    unsafe fn or(a: Self::Vector, b: Self::Vector) -> Self::Vector {
        unsafe { _mm_or_si128(a, b) }
    }

    #[inline]
    #[target_feature(enable = "sse4.2")]
    unsafe fn movemask_epi8(a: Self::Vector) -> u32 {
        unsafe { _mm_movemask_epi8(a) as u32 }
    }
}

/// AVX2 backend (256-bit vectors)
pub struct Avx2;

impl SimdBackend for Avx2 {
    const LANE_WIDTH: usize = 32;
    type Vector = __m256i;

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn load_unaligned(ptr: *const u8) -> Self::Vector {
        unsafe { _mm256_loadu_si256(ptr as *const __m256i) }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn cmpeq_epi8(a: Self::Vector, b: u8) -> Self::Vector {
        unsafe {
            let broadcast = _mm256_set1_epi8(b as i8);
            _mm256_cmpeq_epi8(a, broadcast)
        }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn or(a: Self::Vector, b: Self::Vector) -> Self::Vector {
        unsafe { _mm256_or_si256(a, b) }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn movemask_epi8(a: Self::Vector) -> u32 {
        unsafe { _mm256_movemask_epi8(a) as u32 }
    }
}

/// Scan for structural characters using SSE4.2
///
/// # Safety
/// - Requires SSE4.2 support (caller must verify with `is_x86_feature_detected!("sse4.2")`)
#[target_feature(enable = "sse4.2")]
pub unsafe fn scan_structural_sse42(input: &[u8], indices: &mut Vec<u32>) {
    unsafe { scan_structural_generic::<Sse42>(input, indices) }
}

/// Scan for structural characters using AVX2
///
/// # Safety
/// - Requires AVX2 support (caller must verify with `is_x86_feature_detected!("avx2")`)
#[target_feature(enable = "avx2")]
pub unsafe fn scan_structural_avx2(input: &[u8], indices: &mut Vec<u32>) {
    unsafe { scan_structural_generic::<Avx2>(input, indices) }
}

/// Generic structural scanner using any SimdBackend
///
/// # Safety
/// - Requires the backend's SIMD features to be available
#[inline]
unsafe fn scan_structural_generic<B: SimdBackend>(input: &[u8], indices: &mut Vec<u32>) {
    let len = input.len();
    let mut pos = 0;

    // Process full SIMD chunks
    while pos + B::LANE_WIDTH <= len {
        // SAFETY: We've verified pos + LANE_WIDTH <= len, so pointer arithmetic is valid.
        // The caller must ensure the required SIMD features are available.
        let chunk = unsafe { B::load_unaligned(input.as_ptr().add(pos)) };

        // Check for structural characters: n, t, f, i, l, d, u, :, e, 0-9
        // SAFETY: These operations use the SIMD features guaranteed by the caller.
        let mask_n = unsafe { B::cmpeq_epi8(chunk, b'n') };
        let mask_t = unsafe { B::cmpeq_epi8(chunk, b't') };
        let mask_f = unsafe { B::cmpeq_epi8(chunk, b'f') };
        let mask_i = unsafe { B::cmpeq_epi8(chunk, b'i') };
        let mask_l = unsafe { B::cmpeq_epi8(chunk, b'l') };
        let mask_d = unsafe { B::cmpeq_epi8(chunk, b'd') };
        let mask_u = unsafe { B::cmpeq_epi8(chunk, b'u') };
        let mask_colon = unsafe { B::cmpeq_epi8(chunk, b':') };
        let mask_e = unsafe { B::cmpeq_epi8(chunk, b'e') };

        // Digits 0-9
        let mask_0 = unsafe { B::cmpeq_epi8(chunk, b'0') };
        let mask_1 = unsafe { B::cmpeq_epi8(chunk, b'1') };
        let mask_2 = unsafe { B::cmpeq_epi8(chunk, b'2') };
        let mask_3 = unsafe { B::cmpeq_epi8(chunk, b'3') };
        let mask_4 = unsafe { B::cmpeq_epi8(chunk, b'4') };
        let mask_5 = unsafe { B::cmpeq_epi8(chunk, b'5') };
        let mask_6 = unsafe { B::cmpeq_epi8(chunk, b'6') };
        let mask_7 = unsafe { B::cmpeq_epi8(chunk, b'7') };
        let mask_8 = unsafe { B::cmpeq_epi8(chunk, b'8') };
        let mask_9 = unsafe { B::cmpeq_epi8(chunk, b'9') };

        // Combine all masks
        // SAFETY: These operations use the SIMD features guaranteed by the caller.
        let combined = unsafe {
            B::or(
                B::or(
                    B::or(B::or(mask_n, mask_t), B::or(mask_f, mask_i)),
                    B::or(B::or(mask_l, mask_d), B::or(mask_u, mask_colon)),
                ),
                B::or(
                    B::or(
                        B::or(B::or(mask_e, mask_0), B::or(mask_1, mask_2)),
                        B::or(B::or(mask_3, mask_4), B::or(mask_5, mask_6)),
                    ),
                    B::or(B::or(mask_7, mask_8), mask_9),
                ),
            )
        };

        let mut bits = unsafe { B::movemask_epi8(combined) };

        // Extract positions from bitmask
        while bits != 0 {
            let bit_pos = bits.trailing_zeros();
            indices.push((pos + bit_pos as usize) as u32);
            bits &= bits - 1; // Clear lowest set bit
        }

        pos += B::LANE_WIDTH;
    }

    // Process remaining bytes with scalar code
    while pos < len {
        let byte = input[pos];
        if is_structural_char(byte) {
            indices.push(pos as u32);
        }
        pos += 1;
    }
}

/// Check if a byte is a structural character
#[inline]
fn is_structural_char(b: u8) -> bool {
    matches!(
        b,
        b'n' | b't' | b'f' | b'i' | b'l' | b'd' | b'u' | b':' | b'e' | b'0'..=b'9'
    )
}
