use super::types::*;
use crate::io::{Error as IoError, Write};
use crate::prelude::*;
use core::result::Result;
use num_bigint::BigInt;

/// Encodes a [Bencodex] value into a writer.
///
/// Implementations support both owned and borrowed binary, text, and list
/// values. This trait is not dyn compatible because [`encode`](Self::encode)
/// has a generic writer parameter. Use [`BencodexValue`] to represent values
/// of different Bencodex types together.
///
/// [Bencodex]: https://bencodex.org/
pub trait Encode {
    /// Writes this value in [Bencodex] format to `writer`.
    ///
    /// The writer is not flushed after encoding.
    ///
    /// # Errors
    ///
    /// Returns the writer's I/O error if any bytes cannot be written. The
    /// writer may contain a partial encoding when this method returns an error.
    ///
    /// # Examples
    /// Basic usage with [`BencodexValue::Text`]:
    /// ```
    /// use bencodex::{ Encode, BencodexValue };
    ///
    /// let text = "text".to_string();
    /// let mut vec = Vec::new();
    /// text.encode(&mut vec)?;
    ///
    /// assert_eq!(vec, vec![b'u', b'4', b':', b't', b'e', b'x', b't']);
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    /// [Bencodex]: https://bencodex.org/
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError>;
}

/// Encodes a value into a writer passed by value.
///
/// Pass `&mut writer` to keep using the writer afterward. Borrowed values such
/// as [`str`] and byte slices can be encoded directly without allocating an
/// owned input. The writer is not flushed after encoding.
///
/// # Errors
///
/// Returns the writer's I/O error if any bytes cannot be written. The writer
/// may contain a partial encoding when this function returns an error.
///
/// # Examples
///
/// ```
/// use bencodex::codec::encode::to_writer;
///
/// let mut bytes = Vec::new();
/// to_writer(&mut bytes, "hello")?;
/// assert_eq!(bytes, b"u5:hello");
/// # Ok::<(), bencodex::io::Error>(())
/// ```
pub fn to_writer<W: Write, T: Encode + ?Sized>(mut writer: W, value: &T) -> Result<(), IoError> {
    value.encode(&mut writer)
}

fn write_usize<W: Write>(writer: &mut W, n: usize) -> Result<(), IoError> {
    let mut buf = itoa::Buffer::new();
    let s = buf.format(n);
    writer.write_all(s.as_bytes())
}

fn write_i64<W: Write>(writer: &mut W, n: i64) -> Result<(), IoError> {
    let mut buf = itoa::Buffer::new();
    let s = buf.format(n);
    writer.write_all(s.as_bytes())
}

impl Encode for Vec<u8> {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let buf_val = b"hello".to_vec();
    /// let mut buf = vec![];
    /// buf_val.encode(&mut buf)?;
    /// assert_eq!(buf, b"5:hello");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        self.as_slice().encode(writer)
    }
}

impl Encode for [u8] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        encode_binary(writer, self)
    }
}

impl Encode for i64 {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// 1004i64.encode(&mut buf)?;
    /// assert_eq!(buf, b"i1004e");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"i")?;
        write_i64(writer, *self)?;
        writer.write_all(b"e")
    }
}

impl Encode for String {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// "foo".to_string().encode(&mut buf)?;
    /// assert_eq!(buf, b"u3:foo");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        self.as_str().encode(writer)
    }
}

impl Encode for str {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        encode_text(writer, self)
    }
}

impl Encode for bool {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// true.encode(&mut buf)?;
    /// assert_eq!(buf, b"t");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(if *self { b"t" } else { b"f" })?;

        Ok(())
    }
}

impl Encode for BigInt {
    /// ```
    /// use bencodex::{ Encode };
    /// use num_bigint::BigInt;
    ///
    /// let mut buf = vec![];
    /// BigInt::from(0).encode(&mut buf)?;
    /// assert_eq!(buf, b"i0e");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"i")?;
        writer.write_all(self.to_str_radix(10).as_bytes())?;
        writer.write_all(b"e")?;

        Ok(())
    }
}

impl Encode for Vec<BencodexValue<'_>> {
    /// ```
    /// use bencodex::{ Encode, BencodexValue };
    /// use num_bigint::BigInt;
    ///
    /// let list: Vec<BencodexValue> = vec![0.into(), BencodexValue::Null];
    /// let mut buf = vec![];
    /// list.encode(&mut buf)?;
    /// assert_eq!(buf, b"li0ene");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        self.as_slice().encode(writer)
    }
}

impl Encode for [BencodexValue<'_>] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"l")?;
        for el in self {
            el.encode(writer)?;
        }
        writer.write_all(b"e")?;

        Ok(())
    }
}

fn encode_null<W: Write>(writer: &mut W) -> Result<(), IoError> {
    writer.write_all(b"n")?;

    Ok(())
}

fn encode_binary<W: Write>(writer: &mut W, data: &[u8]) -> Result<(), IoError> {
    write_usize(writer, data.len())?;
    writer.write_all(b":")?;
    writer.write_all(data)?;
    Ok(())
}

fn encode_text<W: Write>(writer: &mut W, text: &str) -> Result<(), IoError> {
    let bytes = text.as_bytes();
    writer.write_all(b"u")?;
    write_usize(writer, bytes.len())?;
    writer.write_all(b":")?;
    writer.write_all(bytes)?;
    Ok(())
}

impl Encode for BencodexValue<'_> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        match self {
            BencodexValue::Binary(x) => x.as_ref().encode(writer)?,
            BencodexValue::Text(x) => x.as_ref().encode(writer)?,
            BencodexValue::Dictionary(x) => x.encode(writer)?,
            BencodexValue::List(x) => x.encode(writer)?,
            BencodexValue::Boolean(x) => x.encode(writer)?,
            BencodexValue::Null => encode_null(writer)?,
            BencodexValue::Number(x) => x.encode(writer)?,
        }

        Ok(())
    }
}

impl Encode for BencodexKey<'_> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        match self {
            BencodexKey::Binary(bytes) => bytes.as_ref().encode(writer),
            BencodexKey::Text(text) => text.as_ref().encode(writer),
        }
    }
}

impl Encode for BTreeMap<BencodexKey<'_>, BencodexValue<'_>> {
    /// ```
    /// use bencodex::{ Encode, BencodexKey, BencodexValue };
    /// use std::collections::BTreeMap;
    ///
    /// let mut dict: BTreeMap<BencodexKey, BencodexValue> = BTreeMap::new();
    /// dict.insert("".into(), "".into());
    ///
    /// let mut buf = vec![];
    /// dict.encode(&mut buf)?;
    ///
    /// assert_eq!(buf, b"du0:u0:e");
    /// # Ok::<(), bencodex::io::Error>(())
    /// ```
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"d")?;
        for (key, value) in self {
            key.encode(writer)?;
            value.encode(writer)?;
        }
        writer.write_all(b"e")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod encode {
        mod btree_map {
            use super::super::super::*;
            use alloc::vec;

            #[test]
            fn should_order_keys() {
                let mut bvalue: BTreeMap<BencodexKey, BencodexValue> = BTreeMap::new();
                bvalue.insert(
                    BencodexKey::Text(Cow::Owned("ua".to_string())),
                    BencodexValue::Null,
                );
                bvalue.insert(
                    BencodexKey::Binary(Cow::Owned(vec![b'a'])),
                    BencodexValue::Null,
                );
                bvalue.insert(
                    BencodexKey::Text(Cow::Owned("ub".to_string())),
                    BencodexValue::Null,
                );
                bvalue.insert(
                    BencodexKey::Binary(Cow::Owned(vec![b'b'])),
                    BencodexValue::Null,
                );

                let mut writer = Vec::new();
                assert!(bvalue.encode(&mut writer).is_ok());
                assert_eq!(b"d1:an1:bnu2:uanu2:ubne".to_vec(), writer);
            }
        }
    }

    #[cfg(feature = "std")]
    mod encode_std {
        struct ConditionFailWriter {
            throw_counts: Vec<u64>,
            call_count: u64,
        }

        impl ConditionFailWriter {
            fn new(throw_counts: Vec<u64>) -> ConditionFailWriter {
                ConditionFailWriter {
                    throw_counts,
                    call_count: 0,
                }
            }
        }

        #[cfg(not(tarpaulin_include))]
        impl std::io::Write for ConditionFailWriter {
            fn write(&mut self, bytes: &[u8]) -> std::result::Result<usize, std::io::Error> {
                self.call_count += 1;
                if self.throw_counts.contains(&self.call_count) {
                    Err(std::io::Error::other(""))
                } else {
                    Ok(bytes.len())
                }
            }

            fn write_all(&mut self, _: &[u8]) -> std::result::Result<(), std::io::Error> {
                self.call_count += 1;
                if self.throw_counts.contains(&self.call_count) {
                    Err(std::io::Error::other(""))
                } else {
                    Ok(())
                }
            }

            fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
                Ok(())
            }
        }

        mod vec_u8 {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue = Vec::<u8>::new();

                // write length
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write bytes
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod btree_map {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let mut bvalue: BTreeMap<BencodexKey, BencodexValue> = BTreeMap::new();
                bvalue.insert(
                    BencodexKey::Text(Cow::Owned("".to_string())),
                    BencodexValue::Null,
                );

                // write 'd'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'u' key prefix
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write '{}' key bytes length
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write ":" key delimeter
                let mut writer = ConditionFailWriter::new(vec![4]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write "" key bytes
                let mut writer = ConditionFailWriter::new(vec![5]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write value
                let mut writer = ConditionFailWriter::new(vec![6]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![7]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod vec_bvalue {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue: Vec<BencodexValue> = vec![BencodexValue::Null];

                // write 'l'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write value
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod string {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue: String = String::new();

                // write 'u'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write length
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write ':'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write text
                let mut writer = ConditionFailWriter::new(vec![4]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod bool {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue = true;

                // write 't'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod big_int {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue = BigInt::from(0);

                // write 'i'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write number
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod i64 {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue: i64 = 0;
                // write 'i'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write number
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }
    }
}
