//! Convert Bencodex values to and from their JSON representation.
//!
//! Available with the `json` feature. Integers are decimal strings, binary
//! values are strings prefixed with `b64:` or `0x`, and text values are strings
//! prefixed with the Unicode byte order mark (`U+FEFF`). Dictionary keys use
//! the same binary and text prefixes. Null, booleans, lists, and dictionaries
//! use the corresponding JSON types.
//!
//! Encoding uses base64 by default; use [`to_json_with_options`] to select hex.
//! Decoding accepts both binary encodings and returns owned values.
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use bencodex::json::{from_json_string, to_json};
//!
//! let value = from_json_string(r#"[null,true,"42","0xff"]"#)?;
//! let encoded = to_json(&value)?;
//! assert_eq!(from_json_string(&encoded)?, value);
//! # Ok(())
//! # }
//! ```

mod decode;
mod encode;

pub use decode::{JsonDecodeError, from_json, from_json_string};
pub use encode::{BinaryEncoding, JsonEncodeOptions, to_json, to_json_with_options};
