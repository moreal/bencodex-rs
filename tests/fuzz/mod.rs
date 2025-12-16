use bencodex::{BencodexKey, BencodexValue};
use num_bigint::BigInt;
use proptest::prelude::*;

#[cfg(feature = "json")]
pub mod json_encode;

pub fn bencodex_key() -> impl Strategy<Value = BencodexKey> {
    prop_oneof![
        prop::collection::vec(any::<u8>(), 0..100).prop_map(BencodexKey::Binary),
        any::<String>().prop_map(BencodexKey::Text),
    ]
}

fn bigint() -> impl Strategy<Value = BigInt> {
    prop::collection::vec(any::<u8>(), 0..32).prop_map(|bytes| {
        if bytes.is_empty() {
            BigInt::from(0)
        } else {
            BigInt::from_signed_bytes_be(&bytes)
        }
    })
}

fn leaf_value() -> impl Strategy<Value = BencodexValue> {
    prop_oneof![
        Just(BencodexValue::Null),
        any::<bool>().prop_map(BencodexValue::Boolean),
        bigint().prop_map(BencodexValue::Number),
        prop::collection::vec(any::<u8>(), 0..100).prop_map(BencodexValue::Binary),
        any::<String>().prop_map(BencodexValue::Text),
    ]
}

pub fn bencodex_value() -> impl Strategy<Value = BencodexValue> {
    leaf_value().prop_recursive(
        4,  // depth
        64, // max nodes
        10, // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..10).prop_map(BencodexValue::List),
                prop::collection::btree_map(bencodex_key(), inner, 0..10)
                    .prop_map(BencodexValue::Dictionary),
            ]
        },
    )
}
