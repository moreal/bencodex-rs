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
//! - **x86_64**: SSE4.2 and AVX2 (runtime detection)
//! - **AArch64**: NEON (always available)
//! - **Other**: Falls back to scalar implementation
//!
//! ## Usage
//!
//! ```ignore
//! use bencodex::simd::decode_simd;
//!
//! let data = b"li42eu5:helloe";
//! let value = decode_simd(data)?;
//! ```

pub mod arch;
pub mod number;
pub mod stage1;
pub mod stage2;
pub mod structural;

use crate::codec::decode::DecodeError;
use crate::codec::types::BencodexValue;

use stage1::build_structural_index;
use stage2::SimdParser;

/// Decode a Bencodex value using SIMD-accelerated parsing.
///
/// This function automatically selects the best available SIMD implementation
/// based on runtime CPU feature detection:
///
/// - On x86_64: Uses AVX2 if available, otherwise SSE4.2
/// - On AArch64: Uses NEON (always available)
/// - On other platforms: Falls back to scalar implementation
///
/// # Arguments
///
/// * `input` - The Bencodex-encoded byte slice to decode
///
/// # Returns
///
/// * `Ok(BencodexValue)` - The decoded value
/// * `Err(DecodeError)` - If the input is not valid Bencodex
///
/// # Example
///
/// ```ignore
/// use bencodex::simd::decode_simd;
///
/// // Decode an integer
/// let value = decode_simd(b"i42e")?;
///
/// // Decode a list
/// let value = decode_simd(b"li1ei2ei3ee")?;
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
}
