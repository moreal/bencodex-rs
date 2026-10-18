use bencodex::{Decode, DecodeError, decode_borrowed};

#[test]
fn duplicate_dictionary_keys_return_an_error() {
    for input in [b"d1:an1:ate".as_slice(), b"du1:anu1:ate"] {
        assert!(decode_borrowed(input).is_err());
        assert!(input.to_vec().decode().is_err());
    }
}

#[test]
fn unrepresentable_binary_range_returns_an_error() {
    let input = format!("{}:", usize::MAX);
    assert!(decode_borrowed(input.as_bytes()).is_err());
    assert!(input.into_bytes().decode().is_err());
}

#[test]
fn unrepresentable_text_range_returns_an_error() {
    let input = format!("u{}:", usize::MAX);
    assert!(decode_borrowed(input.as_bytes()).is_err());
    assert!(input.into_bytes().decode().is_err());
}

#[test]
fn decode_errors_explain_the_token_and_position() {
    let err = decode_borrowed(b"i:e").unwrap_err();
    let message = err.to_string();
    assert!(message.starts_with("unexpected token"));
    assert!(message.contains("0x3a"));
    assert!(message.contains("byte 1"));
    let erased: Box<dyn std::error::Error + Send + Sync> = Box::new(err);
    assert!(erased.downcast_ref::<DecodeError>().is_some());
}
