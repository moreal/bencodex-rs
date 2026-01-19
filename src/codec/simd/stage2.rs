//! Stage 2: Value extraction parser.
//!
//! This stage uses the structural index built in Stage 1 to parse
//! Bencodex values. The structural index allows skipping over data
//! portions without scanning byte-by-byte.

use crate::codec::decode::DecodeError;
use crate::codec::types::*;
use crate::prelude::*;
use core::str;
use num_bigint::BigInt;

use super::structural::StructuralIndex;

/// SIMD-accelerated Bencodex parser.
///
/// Uses a pre-built structural index to parse values more efficiently
/// by jumping directly to structural character positions.
pub struct SimdParser<'a> {
    input: &'a [u8],
    structural: &'a StructuralIndex,
    pos: usize,
    /// Cursor into structural index for efficient lookups
    struct_idx: usize,
}

impl<'a> SimdParser<'a> {
    /// Create a new SIMD parser with the given input and structural index.
    pub fn new(input: &'a [u8], structural: &'a StructuralIndex) -> Self {
        Self {
            input,
            structural,
            pos: 0,
            struct_idx: 0,
        }
    }

    /// Parse a complete Bencodex value from the input.
    pub fn parse(&mut self) -> Result<BencodexValue, DecodeError> {
        self.parse_value()
    }

    /// Parse a single value at the current position.
    fn parse_value(&mut self) -> Result<BencodexValue, DecodeError> {
        if self.pos >= self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }

        match self.input[self.pos] {
            b'd' => self.parse_dict(),
            b'l' => self.parse_list(),
            b'u' => self.parse_unicode_string(),
            b'i' => self.parse_integer(),
            b'0'..=b'9' => self.parse_byte_string(),
            b't' => {
                self.pos += 1;
                Ok(BencodexValue::Boolean(true))
            }
            b'f' => {
                self.pos += 1;
                Ok(BencodexValue::Boolean(false))
            }
            b'n' => {
                self.pos += 1;
                Ok(BencodexValue::Null)
            }
            _ => Err(DecodeError::UnexpectedTokenError {
                token: self.input[self.pos],
                point: self.pos,
            }),
        }
    }

    /// Parse a dictionary: d...e
    fn parse_dict(&mut self) -> Result<BencodexValue, DecodeError> {
        self.expect(b'd')?;
        self.pos += 1;

        let mut map = BTreeMap::new();

        while self.pos < self.input.len() && self.input[self.pos] != b'e' {
            // Parse key (must be binary or text string)
            let key_value = self.parse_value()?;
            let key = match key_value {
                BencodexValue::Text(s) => BencodexKey::Text(s),
                BencodexValue::Binary(b) => BencodexKey::Binary(b),
                _ => return Err(DecodeError::InvalidBencodexValueError),
            };

            // Parse value
            let value = self.parse_value()?;

            map.insert(key, value);
        }

        self.expect(b'e')?;
        self.pos += 1;

        Ok(BencodexValue::Dictionary(map))
    }

    /// Parse a list: l...e
    fn parse_list(&mut self) -> Result<BencodexValue, DecodeError> {
        self.expect(b'l')?;
        self.pos += 1;

        let mut list = Vec::new();

        while self.pos < self.input.len() && self.input[self.pos] != b'e' {
            let value = self.parse_value()?;
            list.push(value);
        }

        self.expect(b'e')?;
        self.pos += 1;

        Ok(BencodexValue::List(list))
    }

    /// Parse a byte string: length:data
    fn parse_byte_string(&mut self) -> Result<BencodexValue, DecodeError> {
        // Find ':' using structural index
        let colon_pos = self
            .find_next_structural(b':')
            .ok_or(DecodeError::InvalidBencodexValueError)?;

        // Parse length from current position to colon
        let length_slice = &self.input[self.pos..colon_pos];
        let length_str =
            str::from_utf8(length_slice).map_err(|_| DecodeError::InvalidBencodexValueError)?;
        let length: usize = length_str
            .parse()
            .map_err(|_| DecodeError::InvalidBencodexValueError)?;

        self.pos = colon_pos + 1;

        // Read data
        if self.pos + length > self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }

        let data = self.input[self.pos..self.pos + length].to_vec();
        self.pos += length;

        Ok(BencodexValue::Binary(data))
    }

    /// Parse a unicode string: ulength:data
    fn parse_unicode_string(&mut self) -> Result<BencodexValue, DecodeError> {
        self.expect(b'u')?;
        self.pos += 1;

        if self.pos >= self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }

        // Find ':' using structural index
        let colon_pos = self
            .find_next_structural(b':')
            .ok_or(DecodeError::InvalidBencodexValueError)?;

        // Parse length from current position to colon
        let length_slice = &self.input[self.pos..colon_pos];
        let length_str =
            str::from_utf8(length_slice).map_err(|_| DecodeError::InvalidBencodexValueError)?;
        let length: usize = length_str
            .parse()
            .map_err(|_| DecodeError::InvalidBencodexValueError)?;

        self.pos = colon_pos + 1;

        // Read data
        if self.pos + length > self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }

        let text = str::from_utf8(&self.input[self.pos..self.pos + length])
            .map_err(|_| DecodeError::InvalidBencodexValueError)?
            .to_string();
        self.pos += length;

        Ok(BencodexValue::Text(text))
    }

    /// Parse an integer: i...e
    fn parse_integer(&mut self) -> Result<BencodexValue, DecodeError> {
        self.expect(b'i')?;
        self.pos += 1;

        if self.pos >= self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }

        // Find 'e' terminator using structural index
        let e_pos = self
            .find_next_structural(b'e')
            .ok_or(DecodeError::InvalidBencodexValueError)?;

        // Parse number between i and e
        let num_slice = &self.input[self.pos..e_pos];
        let num_str =
            str::from_utf8(num_slice).map_err(|_| DecodeError::InvalidBencodexValueError)?;
        let number = num_str
            .parse::<BigInt>()
            .map_err(|_| DecodeError::InvalidBencodexValueError)?;

        self.pos = e_pos + 1;
        Ok(BencodexValue::Number(number))
    }

    /// Expect a specific byte at the current position.
    fn expect(&self, expected: u8) -> Result<(), DecodeError> {
        if self.pos >= self.input.len() {
            return Err(DecodeError::InvalidBencodexValueError);
        }
        if self.input[self.pos] != expected {
            return Err(DecodeError::UnexpectedTokenError {
                token: self.input[self.pos],
                point: self.pos,
            });
        }
        Ok(())
    }

    /// Find next occurrence of `byte` in structural index at or after current position.
    /// Advances struct_idx cursor past the found position.
    fn find_next_structural(&mut self, byte: u8) -> Option<usize> {
        while self.struct_idx < self.structural.len() {
            let pos = self.structural.get(self.struct_idx)? as usize;
            self.struct_idx += 1;

            // Skip positions before current parse position
            if pos < self.pos {
                continue;
            }

            // Check if this position has the byte we're looking for
            if self.input.get(pos) == Some(&byte) {
                return Some(pos);
            }
        }
        None
    }

    /// Reset struct_idx cursor to search from a specific position.
    /// Uses binary search to find first index >= from_pos.
    #[allow(dead_code)]
    fn reset_structural_cursor(&mut self, from_pos: usize) {
        self.struct_idx = self
            .structural
            .indices
            .partition_point(|&p| (p as usize) < from_pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::simd::stage1::build_structural_index;

    fn parse(input: &[u8]) -> Result<BencodexValue, DecodeError> {
        let index = build_structural_index(input);
        let mut parser = SimdParser::new(input, &index);
        parser.parse()
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(parse(b"n").unwrap(), BencodexValue::Null);
    }

    #[test]
    fn test_parse_true() {
        assert_eq!(parse(b"t").unwrap(), BencodexValue::Boolean(true));
    }

    #[test]
    fn test_parse_false() {
        assert_eq!(parse(b"f").unwrap(), BencodexValue::Boolean(false));
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            parse(b"i42e").unwrap(),
            BencodexValue::Number(BigInt::from(42))
        );
        assert_eq!(
            parse(b"i-123e").unwrap(),
            BencodexValue::Number(BigInt::from(-123))
        );
        assert_eq!(
            parse(b"i0e").unwrap(),
            BencodexValue::Number(BigInt::from(0))
        );
    }

    #[test]
    fn test_parse_byte_string() {
        assert_eq!(
            parse(b"5:hello").unwrap(),
            BencodexValue::Binary(b"hello".to_vec())
        );
        assert_eq!(parse(b"0:").unwrap(), BencodexValue::Binary(Vec::new()));
    }

    #[test]
    fn test_parse_unicode_string() {
        assert_eq!(
            parse(b"u5:hello").unwrap(),
            BencodexValue::Text("hello".to_string())
        );
        assert_eq!(parse(b"u0:").unwrap(), BencodexValue::Text(String::new()));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse(b"le").unwrap(), BencodexValue::List(Vec::new()));

        let result = parse(b"li42eu5:helloe").unwrap();
        if let BencodexValue::List(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], BencodexValue::Number(BigInt::from(42)));
            assert_eq!(items[1], BencodexValue::Text("hello".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_dict() {
        assert_eq!(
            parse(b"de").unwrap(),
            BencodexValue::Dictionary(BTreeMap::new())
        );

        let result = parse(b"du1:ai42ee").unwrap();
        if let BencodexValue::Dictionary(map) = result {
            assert_eq!(map.len(), 1);
            assert_eq!(
                map.get(&BencodexKey::Text("a".to_string())),
                Some(&BencodexValue::Number(BigInt::from(42)))
            );
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_parse_nested() {
        // Nested list: [[1, 2], [3]]
        let result = parse(b"lli1ei2eeli3eee").unwrap();
        if let BencodexValue::List(outer) = result {
            assert_eq!(outer.len(), 2);
            if let BencodexValue::List(inner1) = &outer[0] {
                assert_eq!(inner1.len(), 2);
            } else {
                panic!("Expected inner list");
            }
        } else {
            panic!("Expected outer list");
        }
    }
}
