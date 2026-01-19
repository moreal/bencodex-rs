//! Scalar fallback implementation for platforms without SIMD support.

/// Scan for structural characters using scalar code
///
/// This is used as a fallback when SIMD is not available or for
/// processing remainder bytes that don't fill a full SIMD lane.
pub fn scan_structural_scalar(input: &[u8], indices: &mut Vec<u32>) {
    for (pos, &byte) in input.iter().enumerate() {
        if is_structural_char(byte) {
            indices.push(pos as u32);
        }
    }
}

/// Check if a byte is a structural character in Bencodex.
///
/// Structural characters are:
/// - `n`: null
/// - `t`: true
/// - `f`: false
/// - `i`: integer start
/// - `l`: list start
/// - `d`: dictionary start
/// - `u`: unicode string prefix
/// - `:`: separator (after length in strings)
/// - `e`: end marker (for integers, lists, dictionaries)
/// - `0-9`: digits (string length prefix or integer digits)
#[inline]
pub fn is_structural_char(b: u8) -> bool {
    matches!(
        b,
        b'n' | b't' | b'f' | b'i' | b'l' | b'd' | b'u' | b':' | b'e' | b'0'..=b'9'
    )
}
