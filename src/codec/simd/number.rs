//! SIMD-accelerated number parsing utilities.
//!
//! Note: Currently uses scalar parsing as Bencodex integers can be
//! arbitrarily large (BigInt). SIMD optimization would be limited to
//! the digit extraction phase for fixed-size integers.

use num_bigint::BigInt;
use num_traits::ToPrimitive;

/// Fast path for parsing small positive integers (up to 20 digits).
///
/// Returns `None` if the number is too large or negative, in which case
/// the caller should fall back to BigInt parsing.
#[inline]
pub fn try_parse_small_positive(bytes: &[u8]) -> Option<u64> {
    // u64::MAX has 20 digits (18446744073709551615)
    if bytes.is_empty() || bytes.len() > 20 {
        return None;
    }

    // Check for negative sign
    if bytes[0] == b'-' {
        return None;
    }

    let mut result: u64 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as u64)?;
    }

    Some(result)
}

/// Fast path for parsing small integers (positive or negative, up to i64 range).
#[inline]
pub fn try_parse_small_integer(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() {
        return None;
    }

    let is_negative = bytes[0] == b'-';
    let digits = if is_negative { &bytes[1..] } else { bytes };

    // i64::MAX has 19 digits (9223372036854775807)
    // i64::MIN has 19 digits after the sign (-9223372036854775808)
    if digits.is_empty() || digits.len() > 19 {
        return None;
    }

    let unsigned = try_parse_small_positive(digits)?;

    if is_negative {
        // Handle i64::MIN specially to avoid overflow when negating
        // i64::MIN = -9223372036854775808, which is -(i64::MAX + 1)
        let min_abs = (i64::MAX as u64) + 1; // 9223372036854775808
        if unsigned > min_abs {
            return None;
        }
        if unsigned == min_abs {
            return Some(i64::MIN);
        }
        Some(-(unsigned as i64))
    } else {
        if unsigned > i64::MAX as u64 {
            return None;
        }
        Some(unsigned as i64)
    }
}

/// Convert a BigInt to u64 if it fits, otherwise return None.
#[inline]
pub fn bigint_to_u64(n: &BigInt) -> Option<u64> {
    n.to_u64()
}

/// Convert a BigInt to i64 if it fits, otherwise return None.
#[inline]
pub fn bigint_to_i64(n: &BigInt) -> Option<i64> {
    n.to_i64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_small_positive() {
        assert_eq!(try_parse_small_positive(b"0"), Some(0));
        assert_eq!(try_parse_small_positive(b"1"), Some(1));
        assert_eq!(try_parse_small_positive(b"123"), Some(123));
        assert_eq!(
            try_parse_small_positive(b"999999999999999999"),
            Some(999999999999999999)
        );
        // u64::MAX = 18446744073709551615
        assert_eq!(
            try_parse_small_positive(b"18446744073709551615"),
            Some(u64::MAX)
        );

        // Too large (u64::MAX + 1 overflows)
        assert_eq!(try_parse_small_positive(b"18446744073709551616"), None);
        // Way too large (21 digits)
        assert_eq!(try_parse_small_positive(b"999999999999999999999"), None);

        // Negative
        assert_eq!(try_parse_small_positive(b"-1"), None);

        // Empty
        assert_eq!(try_parse_small_positive(b""), None);

        // Non-digit
        assert_eq!(try_parse_small_positive(b"12a3"), None);
    }

    #[test]
    fn test_try_parse_small_integer() {
        assert_eq!(try_parse_small_integer(b"0"), Some(0));
        assert_eq!(try_parse_small_integer(b"123"), Some(123));
        assert_eq!(try_parse_small_integer(b"-123"), Some(-123));
        assert_eq!(
            try_parse_small_integer(b"9223372036854775807"),
            Some(i64::MAX)
        );
        assert_eq!(
            try_parse_small_integer(b"-9223372036854775808"),
            Some(i64::MIN)
        );

        // Too large
        assert_eq!(try_parse_small_integer(b"9223372036854775808"), None);
        assert_eq!(try_parse_small_integer(b"-9223372036854775809"), None);
    }
}
