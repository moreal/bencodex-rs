use super::types::*;
use crate::prelude::*;
use core::error::Error;
use core::fmt;
use core::result::Result;
use core::str;
use core::str::FromStr;
use num_bigint::BigInt;

/// The error type which is returned from decoding a Bencodex value through [`Decode::decode`].
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// This should be used when it failed to decode. In future, it will be separated more and more.
    InvalidBencodexValueError,
    /// This should be used when it failed to decode because there is unexpected token appeared while decoding.
    ///
    /// # Example
    ///
    /// For example, The encoded bytes of [`BencodexValue::Number`] are formed as 'i{}e' (e.g., 'i0e', 'i2147483647e'). If it is not satisified, it should be result through inside [`Err`].
    ///
    /// ```
    /// use bencodex::{ Decode, DecodeError };
    ///
    /// //                     v -- should be b'0' ~ b'9' digit.
    /// let vec = vec![b'i', b':', b'e'];
    /// let error = vec.decode().unwrap_err();
    /// let expected_error = DecodeError::UnexpectedTokenError {
    ///     token: b':',
    ///     point: 1,
    /// };
    /// assert_eq!(expected_error, error);
    /// ```
    UnexpectedTokenError { token: u8, point: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecodeError {}

/// `Decode` is a trait to decode a [Bencodex] value.
///
/// [Bencodex]: https://bencodex.org/
pub trait Decode {
    /// Decodes a [Bencodex] value to return from this type.
    ///
    /// If decoding succeeds, return the value inside [`Ok`]. Otherwise, return the [`DecodeError`] inside [`Err`].
    ///
    /// # Examples
    /// Basic usage with [`Vec<u8>`], the default implementor which implements `Decode`.
    /// ```
    /// use bencodex::{ Decode, BencodexValue };
    ///
    /// let vec = vec![b'n'];
    /// let null = vec.decode().unwrap();
    ///
    /// assert_eq!(BencodexValue::Null, null);
    /// ```
    /// [Bencodex]: https://bencodex.org/
    fn decode(self) -> Result<BencodexValue<'static>, DecodeError>;

    /// Decodes a [Bencodex] value using SIMD-accelerated parsing.
    ///
    /// This method uses SIMD instructions to speed up parsing on supported platforms:
    /// - x86_64: SSE4.2 or AVX2 (runtime detection)
    /// - AArch64: NEON (always available)
    /// - Other platforms: Falls back to scalar implementation
    ///
    /// # Examples
    /// ```ignore
    /// use bencodex::{ Decode, BencodexValue };
    ///
    /// let vec = b"i42e".to_vec();
    /// let number = vec.decode_simd().unwrap();
    /// ```
    /// [Bencodex]: https://bencodex.org/
    #[cfg(feature = "simd")]
    fn decode_simd(self) -> Result<BencodexValue<'static>, DecodeError>;
}

trait ShouldNotBeNone<T> {
    fn should_not_be_none(self) -> Result<T, DecodeError>;
}

impl ShouldNotBeNone<u8> for Option<&u8> {
    #[inline]
    fn should_not_be_none(self) -> Result<u8, DecodeError> {
        match self {
            None => Err(DecodeError::InvalidBencodexValueError),
            Some(v) => Ok(*v),
        }
    }
}

trait Expect<T> {
    fn expect(self, expected: u8, point: usize) -> Result<(), DecodeError>;
}

impl Expect<u8> for u8 {
    #[inline]
    fn expect(self, expected: u8, point: usize) -> Result<(), DecodeError> {
        if self != expected {
            Err(DecodeError::UnexpectedTokenError { token: self, point })
        } else {
            Ok(())
        }
    }
}

fn decode_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    if start >= vector.len() {
        return Err(DecodeError::InvalidBencodexValueError);
    }

    match vector[start] {
        b'd' => decode_dict_impl(vector, start),
        b'l' => decode_list_impl(vector, start),
        b'u' => decode_unicode_string_impl(vector, start),
        b'i' => decode_number_impl(vector, start),
        b'0'..=b'9' => decode_byte_string_impl(vector, start),
        b't' => Ok((BencodexValue::Boolean(true), 1)),
        b'f' => Ok((BencodexValue::Boolean(false), 1)),
        b'n' => Ok((BencodexValue::Null, 1)),
        _ => Err(DecodeError::UnexpectedTokenError {
            token: vector[start],
            point: start,
        }),
    }
}

// start must be on 'd'
fn decode_dict_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    vector
        .get(start)
        .should_not_be_none()?
        .expect(b'd', start)?;

    let mut tsize: usize = 1;
    let mut index = start + tsize;
    let mut map = BTreeMap::new();
    while vector.get(index).should_not_be_none()? != b'e' {
        let (value, size) = decode_impl(vector, index)?;
        let key = match value {
            BencodexValue::Text(s) => BencodexKey::Text(s),
            BencodexValue::Binary(b) => BencodexKey::Binary(b),
            _ => return Err(DecodeError::InvalidBencodexValueError),
        };
        tsize += size;
        index = start + tsize;
        let (value, size) = decode_impl(vector, index)?;

        match map.insert(key, value) {
            None => (),
            Some(_) => todo!(),
        };
        tsize += size;
        index = start + tsize;
    }

    vector
        .get(index)
        .should_not_be_none()?
        .expect(b'e', index)?;
    tsize += 1;

    Ok((BencodexValue::Dictionary(map), tsize))
}

// start must be on 'l'
fn decode_list_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    vector
        .get(start)
        .should_not_be_none()?
        .expect(b'l', start)?;

    let mut tsize: usize = 1;
    let mut list = Vec::new();
    let mut index = start + tsize;
    while vector.get(index).should_not_be_none()? != b'e' {
        let (value, size) = decode_impl(vector, index)?;
        list.push(value);
        tsize += size;
        index = start + tsize
    }

    index = start + tsize;
    vector
        .get(index)
        .should_not_be_none()?
        .expect(b'e', index)?;
    tsize += 1;

    Ok((BencodexValue::List(list), tsize))
}

fn decode_byte_string_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    let mut tsize: usize = 0;
    let (length, size) = match read_length(&vector[start + tsize..]) {
        None => return Err(DecodeError::InvalidBencodexValueError),
        Some(v) => v,
    };
    tsize += size;

    let index = start + tsize;
    vector
        .get(index)
        .should_not_be_none()?
        .expect(b':', index)?;
    tsize += 1;
    if vector.len() < start + tsize + length {
        return Err(DecodeError::InvalidBencodexValueError);
    }
    Ok((
        BencodexValue::Binary(Cow::Borrowed(
            &vector[start + tsize..start + tsize + length],
        )),
        tsize + length,
    ))
}

// start must be on 'u'
fn decode_unicode_string_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    vector
        .get(start)
        .should_not_be_none()?
        .expect(b'u', start)?;

    let mut tsize: usize = 1;
    if vector.len() < start + tsize + 1 {
        return Err(DecodeError::InvalidBencodexValueError);
    }
    let (length, size) = match read_length(&vector[start + tsize..]) {
        None => {
            return Err(DecodeError::UnexpectedTokenError {
                token: vector[start + tsize],
                point: start + tsize,
            });
        }
        Some(v) => v,
    };
    tsize += size;

    let index = start + tsize;
    vector
        .get(index)
        .should_not_be_none()?
        .expect(b':', index)?;
    tsize += 1;

    if vector.len() < start + tsize + length {
        return Err(DecodeError::InvalidBencodexValueError);
    }
    let text = match str::from_utf8(&vector[start + tsize..start + tsize + length]) {
        Ok(v) => v,
        Err(_) => return Err(DecodeError::InvalidBencodexValueError),
    };
    tsize += length;
    Ok((BencodexValue::Text(Cow::Borrowed(text)), tsize))
}

// start must be on 'i'
fn decode_number_impl<'a>(
    vector: &'a [u8],
    start: usize,
) -> Result<(BencodexValue<'a>, usize), DecodeError> {
    let mut tsize: usize = 1;
    if vector.len() < start + tsize + 1 {
        return Err(DecodeError::InvalidBencodexValueError);
    }
    let (number, size) = match read_number(&vector[start + tsize..]) {
        None => {
            return Err(DecodeError::UnexpectedTokenError {
                token: vector[start + tsize],
                point: start + tsize,
            });
        }
        Some(v) => v,
    };
    tsize += size;

    let index = start + tsize;
    vector
        .get(index)
        .should_not_be_none()?
        .expect(b'e', index)?;
    tsize += 1;
    Ok((BencodexValue::Number(number), tsize))
}

/// Parses a non-negative decimal integer from `s` directly into `usize`.
/// Returns `(value, bytes_consumed)` or `None` if the input is empty,
/// starts with `-`, or contains no digits.
#[inline]
fn read_length(s: &[u8]) -> Option<(usize, usize)> {
    if s.is_empty() || s[0] == b'-' {
        return None;
    }

    let mut size: usize = 0;
    let mut value: usize = 0;
    while size < s.len() {
        match s[size] {
            b'0'..=b'9' => {
                value = value
                    .checked_mul(10)?
                    .checked_add((s[size] - b'0') as usize)?;
                size += 1;
            }
            _ => break,
        }
    }

    if size == 0 { None } else { Some((value, size)) }
}

fn read_number(s: &[u8]) -> Option<(BigInt, usize)> {
    if s.is_empty() {
        return None;
    }

    let is_negative = s[0] == b'-';
    if s.len() == 1 && is_negative {
        return None;
    }

    let mut size: usize = is_negative as usize;
    while size < s.len() {
        match s[size] {
            b'0'..=b'9' => {
                size += 1;
                continue;
            }
            _ => break,
        };
    }

    if is_negative && size == 1 || size == 0 {
        None
    } else {
        // Fast-path: small numbers that fit in i64 (up to 19 digits)
        // i64 max is 9,223,372,036,854,775,807 (19 digits)
        let digit_count = if is_negative { size - 1 } else { size };
        // SAFETY: The loop above only advances `size` for bytes matching b'0'..=b'9',
        // and s[0] is checked for b'-'. These are all single-byte ASCII, which is valid UTF-8.
        let str_slice = unsafe { core::str::from_utf8_unchecked(&s[..size]) };
        if digit_count <= 18
            && let Ok(n) = str_slice.parse::<i64>()
        {
            // Safe to parse as i64 (18 digits is always within i64 range)
            return Some((BigInt::from(n), size));
        }
        // Large numbers or parsing edge cases: use BigInt directly
        Some((BigInt::from_str(str_slice).unwrap(), size))
    }
}

/// Decode a Bencodex value with zero-copy borrowing from the input slice.
///
/// The returned value borrows binary and text data directly from `input`,
/// avoiding allocations for those types.
///
/// # Examples
/// ```
/// use bencodex::{BencodexValue, decode_borrowed};
///
/// let data = b"u5:hello";
/// let value = decode_borrowed(data).unwrap();
/// assert_eq!(value, BencodexValue::Text("hello".into()));
/// ```
pub fn decode_borrowed(input: &[u8]) -> Result<BencodexValue<'_>, DecodeError> {
    Ok(decode_impl(input, 0)?.0)
}

impl Decode for Vec<u8> {
    /// ```
    /// use bencodex::{ Decode, BencodexValue };
    /// use std::collections::BTreeMap;
    ///
    /// let buf = b"de".to_vec();
    /// let dictionary = buf.decode().ok().unwrap();
    ///
    /// assert_eq!(dictionary, BencodexValue::Dictionary(BTreeMap::new()));
    /// ```
    fn decode(self) -> Result<BencodexValue<'static>, DecodeError> {
        Ok(decode_impl(&self, 0)?.0.into_owned())
    }

    #[cfg(feature = "simd")]
    fn decode_simd(self) -> Result<BencodexValue<'static>, DecodeError> {
        crate::codec::simd::decode_simd(&self).map(|v| v.into_owned())
    }
}

#[cfg(test)]
mod tests {
    mod decode_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_overflowed_start() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(expected_error, decode_impl(&[], 1).unwrap_err());
            assert_eq!(expected_error, decode_impl(b"12", 2).unwrap_err());
            assert_eq!(expected_error, decode_impl(b"12", 20).unwrap_err());
        }

        #[test]
        fn should_return_unexpected_token_error_with_invalid_source() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'x',
                    point: 0,
                },
                decode_impl(b"x", 0).unwrap_err()
            );
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 4,
                },
                decode_impl(b"xyzok", 4).unwrap_err()
            );
        }
    }

    mod decode_dict_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_insufficient_length_source() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(expected_error, decode_dict_impl(b"d", 0).unwrap_err());
            assert_eq!(expected_error, decode_dict_impl(b"d", 2).unwrap_err());
            assert_eq!(expected_error, decode_dict_impl(&[], 0).unwrap_err());
        }

        #[test]
        fn should_return_error_with_source_having_incorrect_key() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            // { 0: null }
            assert_eq!(expected_error, decode_dict_impl(b"di0ene", 0).unwrap_err());
            // { null: null }
            assert_eq!(expected_error, decode_dict_impl(b"dnne", 0).unwrap_err());
            // { list: null }
            assert_eq!(expected_error, decode_dict_impl(b"dlene", 0).unwrap_err());
            // { dictionary: null }
            assert_eq!(expected_error, decode_dict_impl(b"ddene", 0).unwrap_err());
            // { boolean: null }
            assert_eq!(expected_error, decode_dict_impl(b"dtene", 0).unwrap_err());
        }

        #[test]
        fn should_pass_error() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 1,
                },
                decode_dict_impl(b"dkne", 0).unwrap_err()
            );
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 4,
                },
                decode_dict_impl(b"d1:ake", 0).unwrap_err()
            );
        }
    }

    mod decode_list_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_insufficient_length_source() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(expected_error, decode_list_impl(b"l", 0).unwrap_err());
            assert_eq!(expected_error, decode_list_impl(b"l", 2).unwrap_err());
            assert_eq!(expected_error, decode_list_impl(&[], 0).unwrap_err());
        }

        #[test]
        fn should_pass_error() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 1,
                },
                decode_list_impl(b"lke", 0).unwrap_err()
            );
        }
    }

    mod decode_byte_string_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_insufficient_length_source() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(
                expected_error,
                decode_byte_string_impl(b"1", 0).unwrap_err()
            );
            assert_eq!(
                expected_error,
                decode_byte_string_impl(b"1:", 0).unwrap_err()
            );
            assert_eq!(
                expected_error,
                decode_byte_string_impl(b"2:a", 0).unwrap_err()
            );
            assert_eq!(expected_error, decode_byte_string_impl(&[], 0).unwrap_err());
        }

        #[test]
        fn should_return_unexpected_token_error_with_invalid_source() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 1,
                },
                decode_byte_string_impl(b"1ka", 0).unwrap_err()
            );
        }
    }

    mod decode_unicode_string_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_insufficient_length_source() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(
                expected_error,
                decode_unicode_string_impl(b"u", 0).unwrap_err()
            );
            assert_eq!(
                expected_error,
                decode_unicode_string_impl(b"u1", 0).unwrap_err()
            );
            assert_eq!(
                expected_error,
                decode_unicode_string_impl(b"u2:a", 0).unwrap_err()
            );
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 1,
                },
                decode_unicode_string_impl(b"uk", 0).unwrap_err()
            );
            assert_eq!(
                expected_error,
                decode_unicode_string_impl(&[], 0).unwrap_err()
            );
        }

        #[test]
        fn should_return_unexpected_token_error_with_invalid_source() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'k',
                    point: 2
                },
                decode_unicode_string_impl(b"u1ka", 0).unwrap_err()
            );
        }

        #[test]
        fn should_return_unexpected_token_error_with_negative_length_number() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'-',
                    point: 1,
                },
                decode_unicode_string_impl(b"u-1:a", 0).unwrap_err()
            );
        }

        #[test]
        fn should_return_error_with_invalid_source_having_invalid_unicode_string() {
            assert_eq!(
                DecodeError::InvalidBencodexValueError,
                decode_unicode_string_impl(&[b'u', b'1', b':', 0x90], 0).unwrap_err()
            );
        }
    }

    mod decode_number_impl {
        use super::super::*;

        #[test]
        fn should_return_error_with_insufficient_length_source() {
            let expected_error = DecodeError::InvalidBencodexValueError;
            assert_eq!(expected_error, decode_number_impl(b"i", 0).unwrap_err());
            assert_eq!(expected_error, decode_number_impl(b"i2", 0).unwrap_err());
            assert_eq!(expected_error, decode_number_impl(b"i-2", 0).unwrap_err());
            assert_eq!(expected_error, decode_number_impl(&[], 0).unwrap_err());
        }

        #[test]
        fn should_return_unexpected_token_error_with_invalid_source() {
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'a',
                    point: 1,
                },
                decode_number_impl(b"iaa", 0).unwrap_err()
            );
            assert_eq!(
                DecodeError::UnexpectedTokenError {
                    token: b'a',
                    point: 2,
                },
                decode_number_impl(b"i1a", 0).unwrap_err()
            );
        }
    }

    mod vec_u8 {
        mod decode_impl {
            mod decode {
                use super::super::super::super::*;
                use alloc::vec;

                #[test]
                fn should_pass_error() {
                    assert_eq!(
                        DecodeError::InvalidBencodexValueError,
                        vec![].decode().unwrap_err()
                    );
                    assert_eq!(
                        DecodeError::UnexpectedTokenError {
                            token: b'_',
                            point: 0,
                        },
                        vec![b'_'].decode().unwrap_err()
                    );
                }
            }
        }
    }

    mod u8 {
        mod expect_impl {
            mod expect {
                use super::super::super::super::{DecodeError, Expect};

                #[test]
                fn should_return_unexpected_token_error() {
                    let decode_error = b'a'.expect(b'u', 12).unwrap_err();
                    if let DecodeError::UnexpectedTokenError { token, point } = decode_error {
                        assert_eq!(b'a', token);
                        assert_eq!(12, point);
                    }

                    let decode_error = b'x'.expect(b'u', 100).unwrap_err();
                    if let DecodeError::UnexpectedTokenError { token, point } = decode_error {
                        assert_eq!(b'x', token);
                        assert_eq!(100, point);
                    }
                }
            }
        }
    }

    mod decode_error {
        mod display_impl {
            use super::super::super::*;

            #[test]
            fn fmt() {
                assert_eq!(
                    "InvalidBencodexValueError",
                    DecodeError::InvalidBencodexValueError.to_string()
                )
            }
        }
    }

    mod read_number {
        use super::super::*;

        #[test]
        fn should_return_none() {
            assert_eq!(None, read_number(b""));
        }

        #[test]
        fn should_return_ok_with_positive() {
            assert_eq!(Some((BigInt::from(1), 1)), read_number(b"1"));
            assert_eq!(Some((BigInt::from(326), 3)), read_number(b"326"));
        }

        #[test]
        fn should_return_ok_with_negative() {
            assert_eq!(Some((BigInt::from(-1), 2)), read_number(b"-1"));
            assert_eq!(Some((BigInt::from(-845), 4)), read_number(b"-845"));
        }

        #[test]
        fn should_return_none_with_single_minus_sign() {
            assert_eq!(None, read_number(b"-"));
        }

        #[test]
        fn should_return_none_with_single_minus_sign_and_invalid_char() {
            assert_eq!(None, read_number(b"-e"));
            assert_eq!(None, read_number(b"-x"));
        }
    }

    mod decode_borrowed {
        use super::super::*;

        #[test]
        fn should_return_borrowed_binary() {
            let data = b"5:hello";
            let value = decode_borrowed(data).unwrap();
            assert_eq!(
                value,
                BencodexValue::Binary(Cow::Borrowed(b"hello".as_slice()))
            );
        }

        #[test]
        fn should_return_borrowed_text() {
            let data = b"u5:hello";
            let value = decode_borrowed(data).unwrap();
            assert_eq!(value, BencodexValue::Text(Cow::Borrowed("hello")));
        }

        #[test]
        fn should_match_owned_decode() {
            let data = b"du3:foou3:bare";
            let borrowed = decode_borrowed(data).unwrap().into_owned();
            let owned = data.to_vec().decode().unwrap();
            assert_eq!(borrowed, owned);
        }
    }
}
