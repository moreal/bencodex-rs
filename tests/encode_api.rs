use bencodex::codec::encode::to_writer;
use bencodex::io::{Error, Write};
use bencodex::{BencodexKey, BencodexValue, Encode};

#[test]
fn borrowed_values_encode_without_allocating_owned_inputs() -> Result<(), Error> {
    let mut bytes = Vec::new();
    let binary: &[u8] = b"\x00\xff";
    let list: &[BencodexValue<'_>] = &[BencodexValue::Null, true.into()];

    "한글".encode(&mut bytes)?;
    binary.encode(&mut bytes)?;
    list.encode(&mut bytes)?;

    assert_eq!(bytes, b"u6:\xed\x95\x9c\xea\xb8\x802:\x00\xfflnte");
    Ok(())
}

#[test]
fn keys_encode_as_their_binary_or_text_value() -> Result<(), Error> {
    let mut bytes = Vec::new();
    BencodexKey::from(&b"key"[..]).encode(&mut bytes)?;
    BencodexKey::from("key").encode(&mut bytes)?;
    assert_eq!(bytes, b"3:keyu3:key");
    Ok(())
}

#[test]
fn writer_helper_accepts_a_borrowed_writer_and_unsized_value() -> Result<(), Error> {
    let mut bytes = Vec::new();
    to_writer(&mut bytes, "text")?;
    to_writer(&mut bytes, &b"bytes"[..])?;
    assert_eq!(bytes, b"u4:text5:bytes");
    Ok(())
}

#[test]
fn writer_helper_accepts_an_owned_writer() -> Result<(), Error> {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct SharedWriter(Rc<RefCell<Vec<u8>>>);

    #[cfg(feature = "std")]
    impl Write for SharedWriter {
        fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[cfg(not(feature = "std"))]
    impl Write for SharedWriter {
        fn write_all(&mut self, bytes: &[u8]) -> Result<(), Error> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(())
        }
    }

    let bytes = Rc::new(RefCell::new(Vec::new()));
    to_writer(SharedWriter(Rc::clone(&bytes)), &true)?;
    assert_eq!(*bytes.borrow(), b"t");
    Ok(())
}

#[test]
fn writer_helper_propagates_failures_after_partial_output() {
    struct LimitedWriter(Vec<u8>);

    #[cfg(feature = "std")]
    impl Write for LimitedWriter {
        fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
            if self.0.len() == 2 {
                return Err(Error::other("capacity exhausted"));
            }
            let count = bytes.len().min(2 - self.0.len());
            self.0.extend_from_slice(&bytes[..count]);
            Ok(count)
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[cfg(not(feature = "std"))]
    impl Write for LimitedWriter {
        fn write_all(&mut self, bytes: &[u8]) -> Result<(), Error> {
            let count = bytes.len().min(2 - self.0.len());
            self.0.extend_from_slice(&bytes[..count]);
            if count == bytes.len() {
                Ok(())
            } else {
                Err(Error::new())
            }
        }
    }

    let mut writer = LimitedWriter(Vec::new());
    assert!(to_writer(&mut writer, "abc").is_err());
    assert_eq!(writer.0, b"u3");
}

#[test]
fn downstream_encode_implementations_can_name_the_writer_and_error_types() -> Result<(), Error> {
    struct CustomValue;

    impl Encode for CustomValue {
        fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
            writer.write_all(b"n")
        }
    }

    let mut bytes = Vec::new();
    CustomValue.encode(&mut bytes)?;
    assert_eq!(bytes, b"n");
    Ok(())
}

#[test]
fn borrowed_trait_object_writer_is_supported_by_the_helper() -> Result<(), Error> {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;
    to_writer(writer, &true)?;
    assert_eq!(bytes, b"t");
    Ok(())
}
