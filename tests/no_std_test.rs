#![no_std]

extern crate alloc;
extern crate bencodex;

use alloc::vec::Vec;
use bencodex::{BencodexValue, Decode, Encode};

#[test]
fn test_no_std_encode_decode() {
    // Test basic encoding
    let value = BencodexValue::Number(42.into());
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();
    
    // Test decoding
    let decoded = buf.decode().unwrap();
    assert_eq!(value, decoded);
}