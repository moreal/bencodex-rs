use super::bencodex_value;
use bencodex::json::{to_json, to_json_with_options, BinaryEncoding, JsonEncodeOptions};
use proptest::prelude::*;

proptest! {
    #[test]
    fn json_encode_produces_valid_json(value in bencodex_value()) {
        let json_str = to_json(&value);
        prop_assert!(serde_json::from_str::<serde_json::Value>(&json_str).is_ok());
    }

    #[test]
    fn json_encode_hex_produces_valid_json(value in bencodex_value()) {
        let options = JsonEncodeOptions {
            binary_encoding: BinaryEncoding::Hex,
        };
        let json_str = to_json_with_options(&value, options);
        prop_assert!(serde_json::from_str::<serde_json::Value>(&json_str).is_ok());
    }
}
