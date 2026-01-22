use base64::Engine;
use num_traits::ToPrimitive;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use crate::{BencodexKey, BencodexValue};

struct BencodexJsonEncoder<'a> {
    value: &'a BencodexValue,
    options: JsonEncodeOptions,
}

impl<'a> BencodexJsonEncoder<'a> {
    pub fn new(value: &'a BencodexValue, options: JsonEncodeOptions) -> Self {
        Self { value, options }
    }
}

impl Serialize for BencodexJsonEncoder<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_value(self.value, &self.options, serializer)
    }
}

fn serialize_value<S>(
    value: &BencodexValue,
    options: &JsonEncodeOptions,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        BencodexValue::Null => serializer.serialize_none(),
        BencodexValue::Boolean(b) => serializer.serialize_bool(*b),
        BencodexValue::Number(n) => {
            // Use itoa for small numbers (i64 range) for faster serialization
            if let Some(small) = n.to_i64() {
                let mut buf = itoa::Buffer::new();
                serializer.serialize_str(buf.format(small))
            } else {
                // Large numbers fall back to BigInt::to_string()
                serializer.serialize_str(&n.to_string())
            }
        }
        BencodexValue::Binary(data) => {
            serializer.serialize_str(&format_binary(data, options.binary_encoding))
        }
        BencodexValue::Text(text) => serializer.serialize_str(&format_text(text)),
        BencodexValue::List(items) => {
            let mut seq = serializer.serialize_seq(Some(items.len()))?;
            for item in items {
                seq.serialize_element(&BencodexJsonEncoder::new(item, *options))?;
            }
            seq.end()
        }
        BencodexValue::Dictionary(map) => {
            let mut m = serializer.serialize_map(Some(map.len()))?;
            for (k, v) in map {
                m.serialize_entry(
                    &format_key(k, options),
                    &BencodexJsonEncoder::new(v, *options),
                )?;
            }
            m.end()
        }
    }
}

fn format_key(key: &BencodexKey, options: &JsonEncodeOptions) -> String {
    match key {
        BencodexKey::Binary(data) => format_binary(data, options.binary_encoding),
        BencodexKey::Text(text) => format_text(text),
    }
}

#[inline(always)]
fn format_binary(data: &[u8], encoding: BinaryEncoding) -> String {
    match encoding {
        BinaryEncoding::Base64 => {
            format!(
                "b64:{}",
                base64::engine::general_purpose::STANDARD.encode(data)
            )
        }
        BinaryEncoding::Hex => {
            let mut buf = vec![0u8; data.len() * 2 + 2];
            buf[0] = b'0';
            buf[1] = b'x';
            faster_hex::hex_encode(data, &mut buf[2..]).expect("buffer size is correct");
            // SAFETY: hex_encode produces valid ASCII (0-9, a-f)
            unsafe { String::from_utf8_unchecked(buf) }
        }
    }
}

#[inline(always)]
fn format_text(text: &str) -> String {
    format!("\u{FEFF}{}", text)
}

/// An enum type to choose how to encode Bencodex binary type when encoding to JSON.
#[derive(Default, Copy, Clone)]
pub enum BinaryEncoding {
    #[default]
    Base64,
    Hex,
}

/// Options used by [`to_json_with_options`] when encoding Bencodex to JSON.
///
/// # Examples
///
/// If you want to encode binary as hexadecimal string, you can use like below:
///
/// ```
/// use bencodex::json::{ JsonEncodeOptions, BinaryEncoding };
///
/// JsonEncodeOptions {
///   binary_encoding: BinaryEncoding::Hex,
/// };
/// ```
///
/// If you want to encode binary as base64 string, you can use like below:
///
/// ```
/// use bencodex::json::{ JsonEncodeOptions, BinaryEncoding };
///
/// JsonEncodeOptions {
///   binary_encoding: BinaryEncoding::Base64,
/// };
/// ```
///
/// Or you can use [`JsonEncodeOptions::default`] for base64 case:
///
/// ```
/// use bencodex::json::{ JsonEncodeOptions, BinaryEncoding };
///
/// JsonEncodeOptions::default();
/// ```
#[derive(Default, Copy, Clone)]
pub struct JsonEncodeOptions {
    pub binary_encoding: BinaryEncoding,
}

/// Encode Bencodex to JSON with default options.
pub fn to_json(value: &BencodexValue) -> Result<String, serde_json::Error> {
    // to_json_with_options(value, JsonEncodeOptions::default())
    serde_json::to_string(&BencodexJsonEncoder::new(
        value,
        JsonEncodeOptions::default(),
    ))
}

/// Encode Bencodex to JSON with the given options.
pub fn to_json_with_options(
    value: &BencodexValue,
    options: JsonEncodeOptions,
) -> Result<String, serde_json::Error> {
    // let json_value = encode_value(value, &options);
    // serde_json::to_string(&json_value)
    serde_json::to_string(&BencodexJsonEncoder::new(value, options))
}
