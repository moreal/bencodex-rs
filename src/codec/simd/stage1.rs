//! Stage 1: Structural character scanning.
//!
//! This stage scans the input buffer using SIMD instructions to find
//! all structural characters and build a structural index.

use super::arch::fallback::scan_structural_scalar;
use super::structural::StructuralIndex;

/// Build structural index from input using the best available SIMD implementation.
///
/// This function automatically selects the best SIMD implementation based on
/// runtime CPU feature detection:
/// - AVX2 on x86_64 if available
/// - SSE4.2 on x86_64 as fallback
/// - NEON on AArch64 (always available)
/// - Scalar fallback on other platforms
pub fn build_structural_index(input: &[u8]) -> StructuralIndex {
    // Estimate capacity: structural chars are typically 10-20% of input
    let estimated_capacity = input.len() / 8;
    let mut index = StructuralIndex::with_capacity(estimated_capacity);

    scan_structural(input, &mut index.indices);

    index
}

/// Scan input for structural characters using the best available SIMD.
#[inline]
fn scan_structural(input: &[u8], indices: &mut Vec<u32>) {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: We check CPU features before using SIMD instructions
        unsafe {
            if is_x86_feature_detected!("avx2") {
                super::arch::x86_64::scan_structural_avx2(input, indices);
                return;
            }
            if is_x86_feature_detected!("sse4.2") {
                super::arch::x86_64::scan_structural_sse42(input, indices);
                return;
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is always available on AArch64
        unsafe {
            super::arch::aarch64::scan_structural_neon(input, indices);
            return;
        }
    }

    // Fallback to scalar implementation
    #[allow(unreachable_code)]
    scan_structural_scalar(input, indices);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_structural_index_empty() {
        let index = build_structural_index(b"");
        assert!(index.is_empty());
    }

    #[test]
    fn test_build_structural_index_null() {
        let index = build_structural_index(b"n");
        assert_eq!(index.len(), 1);
        assert_eq!(index.get(0), Some(0));
    }

    #[test]
    fn test_build_structural_index_integer() {
        // i42e - positions: i=0, 4=1, 2=2, e=3
        let index = build_structural_index(b"i42e");
        assert_eq!(index.len(), 4);
        assert_eq!(index.get(0), Some(0)); // i
        assert_eq!(index.get(1), Some(1)); // 4
        assert_eq!(index.get(2), Some(2)); // 2
        assert_eq!(index.get(3), Some(3)); // e
    }

    #[test]
    fn test_build_structural_index_string() {
        // 5:hello - The scanner finds ALL potential structural characters:
        // Position 0: '5' (digit)
        // Position 1: ':' (colon)
        // Position 3: 'e' (matches 'e')
        // Position 4: 'l' (matches 'l')
        // Position 5: 'l' (matches 'l')
        // Note: The parser will ignore chars inside string data
        let index = build_structural_index(b"5:hello");
        assert!(index.len() >= 2);
        assert_eq!(index.get(0), Some(0)); // 5
        assert_eq!(index.get(1), Some(1)); // :
    }

    #[test]
    fn test_build_structural_index_list() {
        // li42en5:helloe - list with integer and string
        let index = build_structural_index(b"li42en5:helloe");
        // l=0, i=1, 4=2, 2=3, e=4, n=5, 5=6, :=7, e=13
        assert!(index.len() >= 9);
    }

    #[test]
    fn test_build_structural_index_dict() {
        // du1:ai42ee - dict with key "a" and value 42
        let index = build_structural_index(b"du1:ai42ee");
        // d=0, u=1, 1=2, :=3, i=5, 4=6, 2=7, e=8, e=9
        assert!(index.len() >= 9);
    }

    #[test]
    fn test_build_structural_index_large_input() {
        // Create a larger input to test SIMD code paths (> 32 bytes for AVX2)
        // Using 'n' (null) which is a single structural char with no ambiguity
        let input: Vec<u8> = (0..100).map(|_| b'n').collect();
        let index = build_structural_index(&input);
        // Each 'n' is 1 structural char
        assert_eq!(index.len(), 100);
    }
}
