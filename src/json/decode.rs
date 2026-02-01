use std::borrow::Cow;
use std::result::Result;

use base64::Engine;
use serde_json::Value;

use crate::{BencodexDictionary, BencodexKey, BencodexValue};

/// The error type which is returned from decoding json to bencodex.
#[derive(Debug, PartialEq)]
pub enum JsonDecodeError {
    /// This should be used when it failed to decode because the given json string is invalid. It is used by [`from_json_string`].
    /// For example, it will be returned when `nulll` string is given.
    InvalidJsonString,
    /// This should be used when it failed to decode because the given json is invalid. It is used by [`from_json`] and [`from_json_string`].
    /// For example, it will be returned when `serde_json::Value::String("0xZZ")` is given.
    InvalidJson,
}

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

/// Decode JSON value to Bencodex value.
///
/// # Examples
///
/// In success case:
///
/// ```
/// use serde_json::from_str;
/// use bencodex::BencodexValue;
/// use bencodex::json::from_json;
///
/// let json = from_str("null").unwrap();
/// let result = from_json(&json);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), BencodexValue::Null);
/// ```
///
/// In error case:
///
/// ```
/// use serde_json::Value;
/// use bencodex::json::{ from_json, JsonDecodeError };
///
/// let result = from_json(&Value::String("0xZZ".to_string()));
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err(), JsonDecodeError::InvalidJson);
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

/// Decode JSON string to Bencodex value.
///
/// # Examples
///
/// In success case:
///
/// ```
/// use bencodex::BencodexValue;
/// use bencodex::json::from_json_string;
///
/// let result = from_json_string("null");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), BencodexValue::Null);
/// ```
///
/// In error case which return [`JsonDecodeError::InvalidJsonString`]:
///
/// ```
/// use bencodex::json::{ from_json_string, JsonDecodeError };
///
/// let result = from_json_string("nulll");
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err(), JsonDecodeError::InvalidJsonString);
/// ```
///
/// In error case which return [`JsonDecodeError::InvalidJson`]:
///
/// ```
/// use bencodex::json::{ from_json_string, JsonDecodeError };
///
/// let result = from_json_string("\"0xZZ\"");
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err(), JsonDecodeError::InvalidJson);
/// ```
pub fn from_json_string(s: &str) -> Result<BencodexValue<'static>, JsonDecodeError> {
    let json = serde_json::from_str(s).map_err(|_| JsonDecodeError::InvalidJsonString)?;
    from_json(&json)
}
