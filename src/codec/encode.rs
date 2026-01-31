use super::types::*;
use crate::io::{Error as IoError, Write};
use crate::prelude::*;
use core::result::Result;
use num_bigint::BigInt;

/// `Encode` is a trait to encode a [Bencodex] value.
///
/// [Bencodex]: https://bencodex.org/
pub trait Encode {
    /// Encode a [Bencodex] value from this type.
    ///
    /// If encoding succeeds, return [`Ok`]. Otherwise, it will pass an I/O error occurred in inner logic.
    ///
    /// # Examples
    /// Basic usage with [`BencodexValue::Text`]:
    /// ```
    /// use bencodex::{ Encode, BencodexValue };
    ///
    /// let text = "text".to_string();
    /// let mut vec = Vec::new();
    /// text.encode(&mut vec);
    ///
    /// assert_eq!(vec, vec![b'u', b'4', b':', b't', b'e', b'x', b't']);
    /// ```
    /// [Bencodex]: https://bencodex.org/
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError>;
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
    /// let mut buf = vec![];
    /// b"hello".to_vec().encode(&mut buf);
    /// assert_eq!(buf, b"5:hello");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        write_usize(writer, self.len())?;
        writer.write_all(b":")?;
        writer.write_all(&self)?;

        Ok(())
    }
}

impl Encode for i64 {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// 1004i64.encode(&mut buf);
    /// assert_eq!(buf, b"i1004e");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"i")?;
        write_i64(writer, self)?;
        writer.write_all(b"e")
    }
}

impl Encode for String {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// "foo".to_string().encode(&mut buf);
    /// assert_eq!(buf, b"u3:foo");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        let bytes = self.into_bytes();
        writer.write_all(b"u")?;
        write_usize(writer, bytes.len())?;
        writer.write_all(b":")?;
        writer.write_all(&bytes)?;

        Ok(())
    }
}

impl Encode for bool {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// true.encode(&mut buf);
    /// assert_eq!(buf, b"t");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(match self {
            true => b"t",
            false => b"f",
        })?;

        Ok(())
    }
}

impl Encode for BigInt {
    /// ```
    /// use bencodex::{ Encode };
    /// use num_bigint::BigInt;
    ///
    /// let mut buf = vec![];
    /// BigInt::from(0).encode(&mut buf);
    /// assert_eq!(buf, b"i0e");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"i")?;
        writer.write_all(self.to_str_radix(10).as_bytes())?;
        writer.write_all(b"e")?;

        Ok(())
    }
}

impl Encode for Vec<BencodexValue> {
    /// ```
    /// use bencodex::{ Encode, BencodexValue };
    /// use num_bigint::BigInt;
    ///
    /// let list: Vec<BencodexValue> = vec![0.into(), BencodexValue::Null];
    /// let mut buf = vec![];
    /// list.encode(&mut buf);
    /// assert_eq!(buf, b"li0ene");
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
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

impl Encode for BencodexValue {
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        // FIXME: rewrite more beautiful.
        match self {
            BencodexValue::Binary(x) => x.encode(writer)?,
            BencodexValue::Text(x) => x.encode(writer)?,
            BencodexValue::Dictionary(x) => x.encode(writer)?,
            BencodexValue::List(x) => x.encode(writer)?,
            BencodexValue::Boolean(x) => x.encode(writer)?,
            BencodexValue::Null => encode_null(writer)?,
            BencodexValue::Number(x) => x.encode(writer)?,
        }

        Ok(())
    }
}

impl Encode for BTreeMap<BencodexKey, BencodexValue> {
    /// ```
    /// use bencodex::{ Encode, BencodexKey, BencodexValue };
    /// use std::collections::BTreeMap;
    ///
    /// let mut dict: BTreeMap<BencodexKey, BencodexValue> = BTreeMap::new();
    /// dict.insert("".into(), "".into());
    ///
    /// let mut buf = vec![];
    /// dict.encode(&mut buf);
    ///
    /// assert_eq!(buf, b"du0:u0:e")
    /// ```
    fn encode<W: Write>(self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(b"d")?;
        for (key, value) in self {
            let key = match key {
                BencodexKey::Binary(x) => BencodexValue::Binary(x),
                BencodexKey::Text(x) => BencodexValue::Text(x),
            };

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
                bvalue.insert(BencodexKey::Text("ua".to_string()), BencodexValue::Null);
                bvalue.insert(BencodexKey::Binary(vec![b'a']), BencodexValue::Null);
                bvalue.insert(BencodexKey::Text("ub".to_string()), BencodexValue::Null);
                bvalue.insert(BencodexKey::Binary(vec![b'b']), BencodexValue::Null);

                let mut writer = Vec::new();
                assert!(bvalue.to_owned().encode(&mut writer).is_ok());
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
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write bytes
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
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
                bvalue.insert(BencodexKey::Text("".to_string()), BencodexValue::Null);

                // write 'd'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'u' key prefix
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write '{}' key bytes length
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write ":" key delimeter
                let mut writer = ConditionFailWriter::new(vec![4]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write "" key bytes
                let mut writer = ConditionFailWriter::new(vec![5]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write value
                let mut writer = ConditionFailWriter::new(vec![6]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![7]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());
            }
        }

        mod vec_bvalue {
            use super::super::super::*;
            use super::*;

            #[test]
            fn should_pass_error() {
                let bvalue: &mut Vec<BencodexValue> = &mut Vec::new();
                bvalue.push(BencodexValue::Null);

                // write 'l'
                let mut writer = ConditionFailWriter::new(vec![1]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write value
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
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
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write length
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write ':'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write text
                let mut writer = ConditionFailWriter::new(vec![4]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
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
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write number
                let mut writer = ConditionFailWriter::new(vec![2]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
                assert_eq!(std::io::ErrorKind::Other, err.kind());
                assert_eq!("", err.to_string());

                // write 'e'
                let mut writer = ConditionFailWriter::new(vec![3]);
                let err = bvalue.to_owned().encode(&mut writer).unwrap_err();
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
