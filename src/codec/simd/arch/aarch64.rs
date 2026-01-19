//! ARM AArch64 SIMD backend (NEON)

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use super::SimdBackend;

/// NEON backend (128-bit vectors)
pub struct Neon;

#[cfg(target_arch = "aarch64")]
impl SimdBackend for Neon {
    const LANE_WIDTH: usize = 16;
    type Vector = uint8x16_t;

    #[inline]
    unsafe fn load_unaligned(ptr: *const u8) -> Self::Vector {
        unsafe { vld1q_u8(ptr) }
    }

    #[inline]
    unsafe fn cmpeq_epi8(a: Self::Vector, b: u8) -> Self::Vector {
        unsafe {
            let broadcast = vdupq_n_u8(b);
            vceqq_u8(a, broadcast)
        }
    }

    #[inline]
    unsafe fn or(a: Self::Vector, b: Self::Vector) -> Self::Vector {
        unsafe { vorrq_u8(a, b) }
    }

    #[inline]
    unsafe fn movemask_epi8(a: Self::Vector) -> u32 {
        // NEON doesn't have a direct movemask equivalent
        // We need to emulate it by extracting the high bit of each byte
        unsafe { neon_movemask(a) }
    }
}

/// Emulate x86 movemask for NEON
///
/// Extracts the most significant bit from each byte in the vector
/// and packs them into a 16-bit result.
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn neon_movemask(v: uint8x16_t) -> u32 {
    unsafe {
        // Shift each byte right by 7 to get the MSB in bit 0 (0x01 for match, 0x00 for no match)
        let shifted = vshrq_n_u8::<7>(v);

        // Create a table of bit positions (1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128)
        let bit_pos =
            vld1q_u8([1u8, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128].as_ptr());

        // Multiply each byte by its bit position weight
        // shifted[i] is 0x01 or 0x00, so multiplication gives us the bit weight or 0
        let masked = vmulq_u8(shifted, bit_pos);

        // Add horizontally
        let low = vget_low_u8(masked);
        let high = vget_high_u8(masked);

        // Sum each 8-byte half
        let low_sum = vaddv_u8(low) as u32;
        let high_sum = vaddv_u8(high) as u32;

        low_sum | (high_sum << 8)
    }
}

/// Scan for structural characters using NEON
///
/// # Safety
/// - Only call on AArch64 platforms (NEON is always available on AArch64)
#[cfg(target_arch = "aarch64")]
pub unsafe fn scan_structural_neon(input: &[u8], indices: &mut Vec<u32>) {
    unsafe {
        scan_structural_generic::<Neon>(input, indices);
    }
}

/// Generic structural scanner using any SimdBackend
#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn scan_structural_generic<B: SimdBackend>(input: &[u8], indices: &mut Vec<u32>) {
    let len = input.len();
    let mut pos = 0;

    // Process full SIMD chunks
    while pos + B::LANE_WIDTH <= len {
        unsafe {
            let chunk = B::load_unaligned(input.as_ptr().add(pos));

            // Check for structural characters: n, t, f, i, l, d, u, :, e, 0-9
            let mask_n = B::cmpeq_epi8(chunk, b'n');
            let mask_t = B::cmpeq_epi8(chunk, b't');
            let mask_f = B::cmpeq_epi8(chunk, b'f');
            let mask_i = B::cmpeq_epi8(chunk, b'i');
            let mask_l = B::cmpeq_epi8(chunk, b'l');
            let mask_d = B::cmpeq_epi8(chunk, b'd');
            let mask_u = B::cmpeq_epi8(chunk, b'u');
            let mask_colon = B::cmpeq_epi8(chunk, b':');
            let mask_e = B::cmpeq_epi8(chunk, b'e');

            // Digits 0-9
            let mask_0 = B::cmpeq_epi8(chunk, b'0');
            let mask_1 = B::cmpeq_epi8(chunk, b'1');
            let mask_2 = B::cmpeq_epi8(chunk, b'2');
            let mask_3 = B::cmpeq_epi8(chunk, b'3');
            let mask_4 = B::cmpeq_epi8(chunk, b'4');
            let mask_5 = B::cmpeq_epi8(chunk, b'5');
            let mask_6 = B::cmpeq_epi8(chunk, b'6');
            let mask_7 = B::cmpeq_epi8(chunk, b'7');
            let mask_8 = B::cmpeq_epi8(chunk, b'8');
            let mask_9 = B::cmpeq_epi8(chunk, b'9');

            // Combine all masks
            let combined = B::or(
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
            );

            let mut bits = B::movemask_epi8(combined);

            // Extract positions from bitmask
            while bits != 0 {
                let bit_pos = bits.trailing_zeros();
                indices.push((pos + bit_pos as usize) as u32);
                bits &= bits - 1; // Clear lowest set bit
            }
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
