use base64::Engine;
use num_traits::ToPrimitive;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use crate::{BencodexKey, BencodexValue};

struct BencodexJsonEncoder<'a> {
    value: &'a BencodexValue<'a>,
    options: JsonEncodeOptions,
}

impl<'a> BencodexJsonEncoder<'a> {
    pub fn new(value: &'a BencodexValue<'a>, options: JsonEncodeOptions) -> Self {
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
    value: &BencodexValue<'_>,
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

fn format_key(key: &BencodexKey<'_>, options: &JsonEncodeOptions) -> String {
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

/// The encoding used for binary values and dictionary keys in JSON.
///
/// Both variants can be decoded by [`super::from_json`] and
/// [`super::from_json_string`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BinaryEncoding {
    /// Standard base64 with padding, prefixed with `b64:` (the default).
    #[default]
    Base64,
    /// Lowercase hexadecimal, prefixed with `0x`.
    Hex,
}

/// Options for encoding Bencodex as JSON with [`to_json_with_options`].
///
/// The default uses [`BinaryEncoding::Base64`].
///
/// # Examples
///
/// ```
/// use bencodex::json::{BinaryEncoding, JsonEncodeOptions};
///
/// let options = JsonEncodeOptions {
///     binary_encoding: BinaryEncoding::Hex,
/// };
/// assert_ne!(options, JsonEncodeOptions::default());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct JsonEncodeOptions {
    /// The encoding used for binary values and dictionary keys.
    pub binary_encoding: BinaryEncoding,
}

/// Encodes a Bencodex value as a compact JSON string using base64 for binary data.
///
/// Uses [`JsonEncodeOptions::default`]. See the
/// [module documentation](crate::json) for the JSON representation.
///
/// # Errors
///
/// Returns any error reported by the JSON serializer. All current Bencodex
/// variants have a supported JSON representation.
///
/// # Examples
///
/// ```
/// use bencodex::BencodexValue;
/// use bencodex::json::to_json;
///
/// let value = BencodexValue::Binary(vec![0xff, 0x00].into());
/// assert_eq!(to_json(&value)?, r#""b64:/wA=""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub fn to_json(value: &BencodexValue<'_>) -> Result<String, serde_json::Error> {
    serde_json::to_string(&BencodexJsonEncoder::new(
        value,
        JsonEncodeOptions::default(),
    ))
}

/// Encodes a Bencodex value as a compact JSON string using the given options.
///
/// See the [module documentation](crate::json) for the JSON representation.
///
/// # Errors
///
/// Returns any error reported by the JSON serializer. All current Bencodex
/// variants have a supported JSON representation.
///
/// # Examples
///
/// ```
/// use bencodex::BencodexValue;
/// use bencodex::json::{BinaryEncoding, JsonEncodeOptions, to_json_with_options};
///
/// let value = BencodexValue::Binary(vec![0xff, 0x00].into());
/// let options = JsonEncodeOptions {
///     binary_encoding: BinaryEncoding::Hex,
/// };
/// assert_eq!(to_json_with_options(&value, options)?, r#""0xff00""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub fn to_json_with_options(
    value: &BencodexValue<'_>,
    options: JsonEncodeOptions,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&BencodexJsonEncoder::new(value, options))
}
