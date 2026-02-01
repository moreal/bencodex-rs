use super::bencodex_value;
use bencodex::simd::decode_simd;
use bencodex::{BencodexValue, Decode, Encode};
use proptest::prelude::*;
use std::borrow::Cow;

fn encode_to_vec(value: &BencodexValue<'_>) -> Vec<u8> {
    let mut buf = Vec::new();
    value.encode(&mut buf).expect("encoding should succeed");
    buf
}

proptest! {
    // 1. Roundtrip test: encode -> decode_simd should return original value
    #[test]
    fn simd_decode_roundtrip(value in bencodex_value()) {
        let encoded = encode_to_vec(&value);
        let decoded = decode_simd(&encoded).expect("should decode").into_owned();
        prop_assert_eq!(value, decoded);
    }

    // 2. Differential test: scalar vs SIMD should produce identical results
    #[test]
    fn simd_vs_scalar_equivalence(value in bencodex_value()) {
        let encoded = encode_to_vec(&value);
        let scalar_result = encoded.clone().decode();
        let simd_result = decode_simd(&encoded).map(|v| v.into_owned());
        prop_assert_eq!(scalar_result, simd_result);
    }

    // 3. Random input: should not panic on arbitrary bytes
    #[test]
    fn simd_no_panic_on_random_input(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = decode_simd(&data);
    }

    // 4. Random input: scalar and SIMD should agree on validity
    #[test]
    fn simd_vs_scalar_on_random_input(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let scalar = data.clone().decode();
        let simd = decode_simd(&data).map(|v| v.into_owned());
        match (&scalar, &simd) {
            (Ok(s), Ok(v)) => prop_assert_eq!(s, v),
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "Mismatch: scalar={:?}, simd={:?}", scalar, simd),
        }
    }
}

// 5. Edge case inputs strategy
fn edge_case_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        Just(vec![]),
        Just(vec![b'n']),
        Just(b"i".to_vec()),
        Just(b"ie".to_vec()),
        Just(b"l".to_vec()),
        Just(b"d".to_vec()),
        Just(vec![0xFF]),
        // SIMD boundary tests (16, 32 bytes)
        prop::collection::vec(any::<u8>(), 15..=17),
        prop::collection::vec(any::<u8>(), 31..=33),
    ]
}

proptest! {
    #[test]
    fn simd_handles_edge_cases(data in edge_case_bytes()) {
        let scalar = data.clone().decode();
        let simd = decode_simd(&data).map(|v| v.into_owned());
        match (&scalar, &simd) {
            (Ok(s), Ok(v)) => prop_assert_eq!(s, v),
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "Mismatch"),
        }
    }
}

// 6. Binary data containing structural characters
fn binary_with_structural_chars() -> impl Strategy<Value = BencodexValue<'static>> {
    prop::collection::vec(
        prop_oneof![
            Just(b'e'),
            Just(b':'),
            Just(b'i'),
            Just(b'l'),
            Just(b'd'),
            any::<u8>()
        ],
        10..500,
    )
    .prop_map(|v| BencodexValue::Binary(Cow::Owned(v)))
}

proptest! {
    #[test]
    fn simd_handles_binary_with_structural_chars(value in binary_with_structural_chars()) {
        let encoded = encode_to_vec(&value);
        let decoded = decode_simd(&encoded).expect("should decode").into_owned();
        prop_assert_eq!(value, decoded);
    }
}
