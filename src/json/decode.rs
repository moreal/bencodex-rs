use std::borrow::Cow;
use std::fmt;
use std::result::Result;

use base64::Engine;
use serde_json::Value;

use crate::{BencodexDictionary, BencodexKey, BencodexValue};

/// An error encountered when decoding the JSON representation of Bencodex.
///
/// Distinguishes invalid JSON syntax from a JSON value that does not represent
/// Bencodex. These categories do not retain an underlying error or location.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JsonDecodeError {
    /// The input to [`from_json_string`] could not be parsed as JSON.
    InvalidJsonString,
    /// A parsed JSON value does not represent Bencodex, for example `"0xZZ"`.
    InvalidJson,
}

impl fmt::Display for JsonDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidJsonString => "invalid json string",
            Self::InvalidJson => "invalid bencodex json value",
        })
    }
}

impl std::error::Error for JsonDecodeError {}

fn from_json_key_impl(s: &str) -> Result<BencodexKey<'static>, JsonDecodeError> {
    if let Some(rest) = s.strip_prefix("b64:") {
        let binary = base64::engine::general_purpose::STANDARD
            .decode(rest)
            .map_err(|_| JsonDecodeError::InvalidJson)?;
        Ok(BencodexKey::Binary(Cow::Owned(binary)))
    } else if let Some(rest) = s.strip_prefix("0x") {
        let binary = hex::decode(rest).map_err(|_| JsonDecodeError::InvalidJson)?;
        Ok(BencodexKey::Binary(Cow::Owned(binary)))
    } else if let Some(rest) = s.strip_prefix('\u{FEFF}') {
        Ok(BencodexKey::Text(Cow::Owned(rest.to_string())))
    } else {
        Err(JsonDecodeError::InvalidJson)
    }
}

/// Decodes a JSON value into an owned Bencodex value.
///
/// See the [module documentation](crate::json) for the JSON representation.
///
/// # Errors
///
/// Returns [`JsonDecodeError::InvalidJson`] if any value or dictionary key
/// does not represent Bencodex. This includes JSON numbers, invalid binary
/// encodings, and strings without a recognized text, binary, or integer form.
///
/// # Examples
///
/// ```
/// use bencodex::BencodexValue;
/// use bencodex::json::from_json;
///
/// let json = serde_json::from_str("null")?;
/// assert_eq!(from_json(&json)?, BencodexValue::Null);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ```
/// use serde_json::Value;
/// use bencodex::json::{ from_json, JsonDecodeError };
///
/// let result = from_json(&Value::String("0xZZ".to_string()));
/// assert_eq!(result, Err(JsonDecodeError::InvalidJson));
/// ```
pub fn from_json(value: &Value) -> Result<BencodexValue<'static>, JsonDecodeError> {
    match value {
        Value::Null => Ok(BencodexValue::Null),
        Value::Bool(b) => Ok(BencodexValue::Boolean(*b)),
        Value::Number(_) => Err(JsonDecodeError::InvalidJson),
        Value::Array(arr) => Ok(BencodexValue::List(
            arr.iter().map(from_json).collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Object(obj) => {
            let mut map = BencodexDictionary::new();
            for (k, v) in obj {
                map.insert(from_json_key_impl(k)?, from_json(v)?);
            }

            Ok(BencodexValue::Dictionary(map))
        }
        Value::String(s) => {
            if let Ok(key) = from_json_key_impl(s) {
                Ok(match key {
                    BencodexKey::Text(t) => BencodexValue::Text(t),
                    BencodexKey::Binary(b) => BencodexValue::Binary(b),
                })
            } else if s
                .as_bytes()
                .iter()
                .all(|x| x.is_ascii_digit() || *x == b'-')
            {
                Ok(BencodexValue::Number(
                    s.parse().map_err(|_| JsonDecodeError::InvalidJson)?,
                ))
            } else {
                Err(JsonDecodeError::InvalidJson)
            }
        }
    }
}

/// Parses a JSON string and decodes it into an owned Bencodex value.
///
/// See the [module documentation](crate::json) for the JSON representation.
///
/// # Errors
///
/// Returns [`JsonDecodeError::InvalidJsonString`] if the input cannot be parsed
/// as JSON. Returns [`JsonDecodeError::InvalidJson`] if a parsed value or key
/// does not represent Bencodex, as described by [`from_json`].
///
/// # Examples
///
/// ```
/// use bencodex::BencodexValue;
/// use bencodex::json::from_json_string;
///
/// assert_eq!(from_json_string("null")?, BencodexValue::Null);
/// # Ok::<(), bencodex::json::JsonDecodeError>(())
/// ```
///
/// Invalid JSON syntax:
///
/// ```
/// use bencodex::json::{ from_json_string, JsonDecodeError };
///
/// let result = from_json_string("nulll");
/// assert_eq!(result, Err(JsonDecodeError::InvalidJsonString));
/// ```
///
/// Valid JSON that does not represent Bencodex:
///
/// ```
/// use bencodex::json::{ from_json_string, JsonDecodeError };
///
/// let result = from_json_string("\"0xZZ\"");
/// assert_eq!(result, Err(JsonDecodeError::InvalidJson));
/// ```
pub fn from_json_string(s: &str) -> Result<BencodexValue<'static>, JsonDecodeError> {
    let json = serde_json::from_str(s).map_err(|_| JsonDecodeError::InvalidJsonString)?;
    from_json(&json)
}
