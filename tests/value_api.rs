use bencodex::{BencodexDictionary, BencodexKey, BencodexValue, Encode};
use num_bigint::BigInt;
use std::borrow::Cow;
use std::collections::HashSet;

#[test]
fn equal_nested_values_deduplicate_across_borrowed_and_owned_storage() {
    let borrowed = BencodexValue::Dictionary(BencodexDictionary::from([
        (
            BencodexKey::from("list"),
            BencodexValue::List(vec![
                BencodexValue::from("text"),
                BencodexValue::from(b"bytes".as_slice()),
                BencodexValue::from(true),
                BencodexValue::from(-7_i32),
                BencodexValue::Null,
            ]),
        ),
        (BencodexKey::from(b"key".as_slice()), BencodexValue::Null),
    ]));
    let owned = borrowed.clone().into_owned();
    let mut values = HashSet::new();
    assert!(values.insert(borrowed));
    assert!(!values.insert(owned));
    assert!(values.insert(BencodexValue::from("bytes")));
    assert!(values.insert(BencodexValue::from(b"bytes".as_slice())));
    assert_eq!(values.len(), 3);
}

#[test]
fn keys_hash_by_kind_and_contents_rather_than_storage() {
    let mut keys = HashSet::new();
    assert!(keys.insert(BencodexKey::from("key")));
    assert!(!keys.insert(BencodexKey::from(String::from("key"))));
    assert!(keys.insert(BencodexKey::from(b"key".as_slice())));
    assert!(!keys.insert(BencodexKey::from(b"key".to_vec())));
    assert_eq!(keys.len(), 2);
}

#[test]
fn wide_and_pointer_sized_integers_preserve_their_exact_value_on_the_wire() {
    fn encoded(value: impl Into<BencodexValue<'static>>) -> Vec<u8> {
        let mut bytes = Vec::new();
        value.into().encode(&mut bytes).expect("encode integer");
        bytes
    }

    assert_eq!(
        encoded(u128::MAX),
        b"i340282366920938463463374607431768211455e"
    );
    assert_eq!(
        encoded(i128::MIN),
        b"i-170141183460469231731687303715884105728e"
    );
    assert_eq!(encoded(123_usize), b"i123e");
    assert_eq!(encoded(-123_isize), b"i-123e");
    let beyond_u128 = BigInt::from(u128::MAX) + BigInt::from(1_u8);
    assert_eq!(
        encoded(beyond_u128),
        b"i340282366920938463463374607431768211456e"
    );
}

#[test]
fn key_to_value_conversion_preserves_borrowing_and_owned_allocations() {
    let text = String::from("text");
    let borrowed: BencodexValue<'_> = BencodexKey::from(text.as_str()).into();
    assert!(matches!(
        borrowed,
        BencodexValue::Text(Cow::Borrowed("text"))
    ));

    let bytes = vec![0_u8, 1, 255];
    let borrowed: BencodexValue<'_> = BencodexKey::from(bytes.as_slice()).into();
    assert!(matches!(
        borrowed,
        BencodexValue::Binary(Cow::Borrowed([0, 1, 255]))
    ));

    let owned_text = String::from("owned text");
    let text_pointer = owned_text.as_ptr();
    let owned: BencodexValue<'static> = BencodexKey::from(owned_text).into();
    match owned {
        BencodexValue::Text(Cow::Owned(text)) => assert_eq!(text.as_ptr(), text_pointer),
        other => panic!("expected owned text, got {other:?}"),
    }

    let owned_bytes = vec![0_u8, 255];
    let bytes_pointer = owned_bytes.as_ptr();
    let owned: BencodexValue<'static> = BencodexKey::from(owned_bytes).into();
    match owned {
        BencodexValue::Binary(Cow::Owned(bytes)) => assert_eq!(bytes.as_ptr(), bytes_pointer),
        other => panic!("expected owned bytes, got {other:?}"),
    }
}

#[test]
fn byte_vectors_remain_binary_while_other_integer_vectors_are_lists() {
    let mut bytes = Vec::new();
    BencodexValue::from(vec![1_u8, 2, 3])
        .encode(&mut bytes)
        .expect("encode binary");
    assert_eq!(bytes, b"3:\x01\x02\x03");

    bytes.clear();
    BencodexValue::from(vec![1_u128, 2, 3])
        .encode(&mut bytes)
        .expect("encode list");
    assert_eq!(bytes, b"li1ei2ei3ee");
}
