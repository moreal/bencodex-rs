#![cfg(feature = "json")]

use std::collections::HashSet;
use std::error::Error;

use bencodex::BencodexValue;
use bencodex::json::{
    BinaryEncoding, JsonDecodeError, JsonEncodeOptions, from_json_string, to_json_with_options,
};

#[test]
fn decode_errors_propagate_through_application_errors() {
    fn decode(input: &str) -> Result<BencodexValue<'static>, Box<dyn Error>> {
        Ok(from_json_string(input)?)
    }

    for (input, expected) in [
        ("nulll", JsonDecodeError::InvalidJsonString),
        ("123", JsonDecodeError::InvalidJson),
        (r#""0xZZ""#, JsonDecodeError::InvalidJson),
    ] {
        let error = decode(input).unwrap_err();
        assert_eq!(error.downcast_ref::<JsonDecodeError>(), Some(&expected));
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
    assert_ne!(
        JsonDecodeError::InvalidJsonString.to_string(),
        JsonDecodeError::InvalidJson.to_string(),
    );
}

#[test]
fn decode_error_categories_can_be_copied_and_collected() {
    fn copied<T: Copy>(value: T) -> T {
        value
    }

    let invalid = JsonDecodeError::InvalidJson;
    let mut categories = HashSet::new();
    categories.insert(copied(invalid));
    categories.insert(invalid);
    categories.insert(JsonDecodeError::InvalidJsonString);
    assert_eq!(categories.len(), 2);
}

#[test]
fn encoding_options_support_comparison_and_diagnostics() -> Result<(), Box<dyn Error>> {
    let default = JsonEncodeOptions::default();
    let explicit_base64 = JsonEncodeOptions {
        binary_encoding: BinaryEncoding::Base64,
    };
    let hex = JsonEncodeOptions {
        binary_encoding: BinaryEncoding::Hex,
    };
    assert_eq!(default, explicit_base64);
    assert_ne!(default, hex);
    assert_eq!(default.binary_encoding, BinaryEncoding::Base64);
    assert!(format!("{hex:?}").contains("Hex"));
    assert_eq!(HashSet::from([default, explicit_base64, hex]).len(), 2);

    let value = BencodexValue::Binary(vec![0xff, 0x00].into());
    assert_eq!(to_json_with_options(&value, default)?, r#""b64:/wA=""#);
    assert_eq!(to_json_with_options(&value, hex)?, r#""0xff00""#);
    Ok(())
}
