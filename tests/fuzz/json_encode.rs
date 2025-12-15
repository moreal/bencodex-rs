use super::ArbitraryBencodexValue;
use bencodex::json::{to_json, to_json_with_options, BinaryEncoding, JsonEncodeOptions};
use quickcheck_macros::quickcheck;

#[quickcheck]
fn json_encode_produces_valid_json(value: ArbitraryBencodexValue) -> bool {
    let json_str = to_json(&value.0);
    serde_json::from_str::<serde_json::Value>(&json_str).is_ok()
}

#[quickcheck]
fn json_encode_hex_produces_valid_json(value: ArbitraryBencodexValue) -> bool {
    let options = JsonEncodeOptions {
        binary_encoding: BinaryEncoding::Hex,
    };
    let json_str = to_json_with_options(&value.0, options);
    serde_json::from_str::<serde_json::Value>(&json_str).is_ok()
}
