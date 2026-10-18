//! SIMD-accelerated Bencodex decoder.
//!
//! This module provides a SIMD-optimized decoder that uses a two-stage parsing
//! strategy inspired by simdjson:
//!
//! 1. **Stage 1 (Structural Scanning)**: Uses SIMD instructions to scan the input
//!    and identify all structural characters (type markers, delimiters, digits).
//!    This builds a "structural index" of positions.
//!
//! 2. **Stage 2 (Value Extraction)**: Uses the structural index to parse values
//!    more efficiently by jumping directly to relevant positions.
//!
//! ## Supported Architectures
//!
//! - **x86_64**: AVX2 or SSE4.2 (runtime detection), with a scalar fallback
//! - **AArch64**: NEON (always available)
//! - **Other**: Falls back to scalar implementation
//!
//! ## Usage
//!
//! ```
//! use bencodex::simd::decode_simd;
//!
//! # fn main() -> Result<(), bencodex::DecodeError> {
//! let data = b"li42eu5:helloe";
//! let value = decode_simd(data)?;
//! # Ok(())
//! # }
//! ```

mod arch;
mod stage1;
mod stage2;
mod structural;

use crate::codec::decode::DecodeError;
use crate::codec::types::BencodexValue;

use stage1::build_structural_index;
use stage2::SimdParser;

/// Decode the first Bencodex value using SIMD-accelerated parsing.
///
/// This function automatically selects the best available SIMD implementation
/// based on runtime CPU feature detection:
///
/// - On x86_64: Uses AVX2 if available, then SSE4.2, then scalar code
/// - On AArch64: Uses NEON (always available)
/// - On other platforms: Falls back to scalar implementation
///
/// Binary and text payloads, including dictionary keys, borrow from `input`.
/// Containers and arbitrary-precision integers allocate their own storage.
///
/// Trailing bytes are ignored. Successful decoding does not certify canonical
/// Bencodex: this parser accepts forms such as leading zeros and unsorted
/// dictionary keys. It rejects duplicate keys. Structural positions use 32-bit
/// offsets, so inputs larger than `u32::MAX` bytes are unsupported.
///
/// # Errors
///
/// Returns [`DecodeError::UnexpectedTokenError`] for an unexpected type marker.
/// Returns [`DecodeError::InvalidBencodexValueError`] for an empty or truncated
/// value, invalid integer or string length, invalid UTF-8 text, duplicate
/// dictionary keys, or a dictionary key that is neither binary nor text.
///
/// # Example
///
/// ```
/// use bencodex::simd::decode_simd;
/// use bencodex::BencodexValue;
///
/// # fn main() -> Result<(), bencodex::DecodeError> {
/// // Decode an integer
/// let value = decode_simd(b"i42e")?;
/// assert!(matches!(value, BencodexValue::Number(_)));
///
/// // Decode a list
/// let value = decode_simd(b"li1ei2ei3ee")?;
/// assert!(matches!(value, BencodexValue::List(items) if items.len() == 3));
/// # Ok(())
/// # }
/// ```
pub fn decode_simd<'a>(input: &'a [u8]) -> Result<BencodexValue<'a>, DecodeError> {
    // Stage 1: Build structural index using SIMD
    let structural_index = build_structural_index(input);

    // Stage 2: Parse using the structural index
    let mut parser = SimdParser::new(input, &structural_index);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::{BencodexDictionary, BencodexKey};
    use alloc::borrow::Cow;
    use num_bigint::BigInt;

    #[test]
    fn test_decode_simd_null() {
        assert_eq!(decode_simd(b"n").unwrap(), BencodexValue::Null);
    }

    #[test]
    fn test_decode_simd_boolean() {
        assert_eq!(decode_simd(b"t").unwrap(), BencodexValue::Boolean(true));
        assert_eq!(decode_simd(b"f").unwrap(), BencodexValue::Boolean(false));
    }

    #[test]
    fn test_decode_simd_integer() {
        assert_eq!(
            decode_simd(b"i0e").unwrap(),
            BencodexValue::Number(BigInt::from(0))
        );
        assert_eq!(
            decode_simd(b"i42e").unwrap(),
            BencodexValue::Number(BigInt::from(42))
        );
        assert_eq!(
            decode_simd(b"i-42e").unwrap(),
            BencodexValue::Number(BigInt::from(-42))
        );
    }

    #[test]
    fn test_decode_simd_binary() {
        assert_eq!(
            decode_simd(b"0:").unwrap(),
            BencodexValue::Binary(Cow::Borrowed(&[]))
        );
        assert_eq!(
            decode_simd(b"5:hello").unwrap(),
            BencodexValue::Binary(Cow::Borrowed(b"hello".as_slice()))
        );
    }

    #[test]
    fn test_decode_simd_text() {
        assert_eq!(
            decode_simd(b"u0:").unwrap(),
            BencodexValue::Text(Cow::Borrowed(""))
        );
        assert_eq!(
            decode_simd(b"u5:hello").unwrap(),
            BencodexValue::Text(Cow::Borrowed("hello"))
        );
    }

    #[test]
    fn test_decode_simd_list() {
        assert_eq!(
            decode_simd(b"le").unwrap(),
            BencodexValue::List(alloc::vec::Vec::new())
        );
        assert_eq!(
            decode_simd(b"li1ei2ei3ee").unwrap(),
            BencodexValue::List(alloc::vec![
                BencodexValue::Number(BigInt::from(1)),
                BencodexValue::Number(BigInt::from(2)),
                BencodexValue::Number(BigInt::from(3)),
            ])
        );
    }

    #[test]
    fn test_decode_simd_dict() {
        assert_eq!(
            decode_simd(b"de").unwrap(),
            BencodexValue::Dictionary(BencodexDictionary::new())
        );

        let result = decode_simd(b"du1:ai42ee").unwrap();
        if let BencodexValue::Dictionary(map) = result {
            assert_eq!(
                map.get(&BencodexKey::Text(Cow::Borrowed("a"))),
                Some(&BencodexValue::Number(BigInt::from(42)))
            );
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_decode_simd_error() {
        assert!(decode_simd(b"").is_err());
        assert!(decode_simd(b"x").is_err());
        assert!(decode_simd(b"i42").is_err()); // Missing 'e'
    }

    #[test]
    fn rejects_binary_length_that_overflows_input_range() {
        let input = alloc::format!("{}:", usize::MAX);
        assert_eq!(
            decode_simd(input.as_bytes()),
            Err(DecodeError::InvalidBencodexValueError)
        );
    }

    #[test]
    fn rejects_text_length_that_overflows_input_range() {
        let input = alloc::format!("u{}:", usize::MAX);
        assert_eq!(
            decode_simd(input.as_bytes()),
            Err(DecodeError::InvalidBencodexValueError)
        );
    }

    #[test]
    fn rejects_duplicate_dictionary_keys() {
        for input in [b"d1:an1:ate".as_slice(), b"du1:anu1:ate"] {
            assert_eq!(
                decode_simd(input),
                Err(DecodeError::InvalidBencodexValueError)
            );
        }
    }
}
