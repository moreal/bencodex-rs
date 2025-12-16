use base64::Engine;
use serde_json::{Map, Value};

use crate::{BencodexKey, BencodexValue};

fn format_key(key: &BencodexKey, options: &JsonEncodeOptions) -> String {
    match key {
        BencodexKey::Binary(data) => match options.binary_encoding {
            BinaryEncoding::Base64 => {
                format!(
                    "b64:{}",
                    base64::engine::general_purpose::STANDARD.encode(data)
                )
            }
            BinaryEncoding::Hex => format!("0x{}", hex::encode(data)),
        },
        BencodexKey::Text(text) => format!("\u{FEFF}{}", text),
    }
}

fn encode_value(value: &BencodexValue, options: &JsonEncodeOptions) -> Value {
    match value {
        BencodexValue::Null => Value::Null,
        BencodexValue::Boolean(b) => Value::Bool(*b),
        BencodexValue::Number(n) => Value::String(n.to_string()),
        BencodexValue::Binary(data) => {
            Value::String(format_key(&BencodexKey::Binary(data.clone()), options))
        }
        BencodexValue::Text(text) => {
            Value::String(format_key(&BencodexKey::Text(text.clone()), options))
        }
        BencodexValue::List(items) => {
            Value::Array(items.iter().map(|v| encode_value(v, options)).collect())
        }
        BencodexValue::Dictionary(map) => {
            let obj: Map<String, Value> = map
                .iter()
                .map(|(k, v)| (format_key(k, options), encode_value(v, options)))
                .collect();
            Value::Object(obj)
        }
    }
}

/// An enum type to choose how to encode Bencodex binary type when encoding to JSON.
#[derive(Default)]
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
#[derive(Default)]
pub struct JsonEncodeOptions {
    pub binary_encoding: BinaryEncoding,
}

/// Encode Bencodex to JSON with default options.
pub fn to_json(value: &BencodexValue) -> String {
    to_json_with_options(value, JsonEncodeOptions::default())
}

/// Encode Bencodex to JSON with the given options.
pub fn to_json_with_options(value: &BencodexValue, options: JsonEncodeOptions) -> String {
    let json_value = encode_value(value, &options);
    serde_json::to_string(&json_value).unwrap()
}
