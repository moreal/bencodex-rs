use super::types::*;
use itertools::Itertools;
use num_bigint::BigInt;

#[cfg(feature = "std")]
use std::cmp::Ordering;
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::result::Result;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::cmp::Ordering;
#[cfg(not(feature = "std"))]
use core::result::Result;

#[cfg(feature = "std")]
/// `Encode` is a trait to encode a [Bencodex] value.
///
/// [Bencodex]: https://bencodex.org/
pub trait Encode {
    /// Encode a [Bencodex] value from this type.
    ///
    /// If encoding succeeds, return [`Ok`]. Otherwise, it will pass [`std::io::Error`] occurred in inner logic.
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
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error>;
}

#[cfg(not(feature = "std"))]
pub trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError>;

    /// Write an entire buffer into this writer.
    fn write_all(&mut self, buf: &[u8]) -> Result<(), WriteError>;

    /// Write a formatted string into this writer.
    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<(), WriteError>;
}

#[cfg(not(feature = "std"))]
/// Error type for write operations
#[derive(Debug)]
pub struct WriteError;

#[cfg(not(feature = "std"))]
impl core::fmt::Display for WriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "WriteError")
    }
}

#[cfg(not(feature = "std"))]
impl Write for alloc::vec::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), WriteError> {
        self.extend_from_slice(buf);
        Ok(())
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<(), WriteError> {
        // Implementation inspired by the std library
        struct Adapter<'a, 'b> {
            inner: &'a mut Vec<u8>,
            error: &'b mut Result<(), WriteError>,
        }

        impl<'a, 'b> core::fmt::Write for Adapter<'a, 'b> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        *self.error = Err(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }

        let mut error = Ok(());
        let result = {
            let mut adapter = Adapter {
                inner: self,
                error: &mut error,
            };
            core::fmt::write(&mut adapter, fmt)
        };

        if result.is_err() {
            if error.is_ok() {
                return Err(WriteError);
            }
            return error;
        }

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
/// `Encode` is a trait to encode a [Bencodex] value.
///
/// [Bencodex]: https://bencodex.org/
pub trait Encode {
    /// Encode a [Bencodex] value from this type.
    ///
    /// If encoding succeeds, return [`Ok`]. Otherwise, it will pass [`WriteError`] occurred in inner logic.
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
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError>;
}

#[cfg(feature = "std")]
impl Encode for Vec<u8> {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// b"hello".to_vec().encode(&mut buf);
    /// assert_eq!(buf, b"5:hello");
    /// ```
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        write!(writer, "{}:", self.len())?;
        writer.write_all(&self)?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl Encode for Vec<u8> {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// b"hello".to_vec().encode(&mut buf);
    /// assert_eq!(buf, b"5:hello");
    /// ```
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        write!(writer, "{}:", self.len())?;
        writer.write_all(&self)?;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl Encode for i64 {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// 1004i64.encode(&mut buf);
    /// assert_eq!(buf, b"i1004e");
    /// ```
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        write!(writer, "i{}e", self)
    }
}

#[cfg(not(feature = "std"))]
impl Encode for i64 {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// 1004i64.encode(&mut buf);
    /// assert_eq!(buf, b"i1004e");
    /// ```
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        write!(writer, "i{}e", self)
    }
}

#[cfg(feature = "std")]
impl Encode for String {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// "foo".to_string().encode(&mut buf);
    /// assert_eq!(buf, b"u3:foo");
    /// ```
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        let bytes = self.into_bytes();
        write!(writer, "u{}:", bytes.len())?;
        writer.write_all(&bytes)?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl Encode for String {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// "foo".to_string().encode(&mut buf);
    /// assert_eq!(buf, b"u3:foo");
    /// ```
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        let bytes = self.into_bytes();
        write!(writer, "u{}:", bytes.len())?;
        writer.write_all(&bytes)?;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl Encode for bool {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// true.encode(&mut buf);
    /// assert_eq!(buf, b"t");
    /// ```
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        writer.write_all(match self {
            true => &[b't'],
            false => &[b'f'],
        })?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl Encode for bool {
    /// ```
    /// use bencodex::{ Encode };
    ///
    /// let mut buf = vec![];
    /// true.encode(&mut buf);
    /// assert_eq!(buf, b"t");
    /// ```
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        writer.write_all(match self {
            true => &[b't'],
            false => &[b'f'],
        })?;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl Encode for BigInt {
    /// ```
    /// use bencodex::{ Encode };
    /// use num_bigint::BigInt;
    ///
    /// let mut buf = vec![];
    /// BigInt::from(0).encode(&mut buf);
    /// assert_eq!(buf, b"i0e");
    /// ```
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        writer.write_all(&[b'i'])?;
        writer.write_all(&self.to_str_radix(10).into_bytes())?;
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl Encode for BigInt {
    /// ```
    /// use bencodex::{ Encode };
    /// use num_bigint::BigInt;
    ///
    /// let mut buf = vec![];
    /// BigInt::from(0).encode(&mut buf);
    /// assert_eq!(buf, b"i0e");
    /// ```
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        writer.write_all(&[b'i'])?;
        writer.write_all(&self.to_str_radix(10).into_bytes())?;
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(feature = "std")]
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
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        writer.write_all(&[b'l'])?;
        for el in self {
            el.encode(writer)?;
        }
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
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
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        writer.write_all(&[b'l'])?;
        for el in self {
            el.encode(writer)?;
        }
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(feature = "std")]
fn encode_null(writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
    writer.write_all(&[b'n'])?;

    Ok(())
}

#[cfg(not(feature = "std"))]
fn encode_null(writer: &mut dyn Write) -> Result<(), WriteError> {
    writer.write_all(&[b'n'])?;

    Ok(())
}

#[cfg(feature = "std")]
impl Encode for BencodexValue {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
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

#[cfg(not(feature = "std"))]
impl Encode for BencodexValue {
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
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

fn compare_vector<T: Ord>(xs: &[T], ys: &[T]) -> Ordering {
    for (x, y) in xs.iter().zip(ys) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
        };
    }

    xs.len().cmp(&ys.len())
}

fn compare_key(x: &BencodexKey, y: &BencodexKey) -> Ordering {
    match (x, y) {
        (BencodexKey::Text(x), BencodexKey::Text(y)) => compare_vector(x.as_bytes(), y.as_bytes()),
        (BencodexKey::Binary(x), BencodexKey::Binary(y)) => compare_vector(x, y),
        (BencodexKey::Text(_), BencodexKey::Binary(_)) => Ordering::Greater,
        (BencodexKey::Binary(_), BencodexKey::Text(_)) => Ordering::Less,
    }
}

#[cfg(feature = "std")]
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
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        let pairs = self
            .into_iter()
            .sorted_by(|(x, _), (y, _)| compare_key(x, y));

        writer.write_all(&[b'd'])?;
        for (key, value) in pairs {
            let key = match key {
                BencodexKey::Binary(x) => BencodexValue::Binary(x),
                BencodexKey::Text(x) => BencodexValue::Text(x),
            };

            key.encode(writer)?;
            value.encode(writer)?;
        }
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(not(feature = "std"))]
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
    fn encode(self, writer: &mut dyn Write) -> Result<(), WriteError> {
        let pairs = self
            .into_iter()
            .sorted_by(|(x, _), (y, _)| compare_key(x, y));

        writer.write_all(&[b'd'])?;
        for (key, value) in pairs {
            let key = match key {
                BencodexKey::Binary(x) => BencodexValue::Binary(x),
                BencodexKey::Text(x) => BencodexValue::Text(x),
            };

            key.encode(writer)?;
            value.encode(writer)?;
        }
        writer.write_all(&[b'e'])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod compare_key {
        use super::super::*;

        #[test]
        fn should_return_equal() {
            assert_eq!(
                Ordering::Equal,
                compare_key(
                    &BencodexKey::Text("foo".to_string()),
                    &BencodexKey::Text("foo".to_string())
                )
            );
            assert_eq!(
                Ordering::Equal,
                compare_key(
                    &BencodexKey::Binary(b"bar".to_vec()),
                    &BencodexKey::Binary(b"bar".to_vec())
                )
            );
        }

        #[test]
        fn should_return_greater() {
            assert_eq!(
                Ordering::Greater,
                compare_key(
                    &BencodexKey::Text("".to_string()),
                    &BencodexKey::Binary(b"".to_vec())
                )
            );
        }

        #[test]
        fn should_return_less() {
            assert_eq!(
                Ordering::Less,
                compare_key(
                    &BencodexKey::Binary(b"".to_vec()),
                    &BencodexKey::Text("".to_string())
                )
            );
        }
    }

    mod compare_vector {
        use super::super::*;

        #[test]
        fn should_return_equal() {
            assert_eq!(
                Ordering::Equal,
                compare_vector(&Vec::<u8>::new(), &Vec::<u8>::new())
            );
            assert_eq!(
                Ordering::Equal,
                compare_vector(&vec![1, 2, 3], &vec![1, 2, 3])
            );
        }

        #[test]
        fn should_return_less() {
            assert_eq!(Ordering::Less, compare_vector(&vec![], &vec![3]));
            assert_eq!(Ordering::Less, compare_vector(&vec![0], &vec![1, 2, 3]));
            assert_eq!(Ordering::Less, compare_vector(&vec![1], &vec![9, 1, 1]));
            assert_eq!(Ordering::Less, compare_vector(&vec![1, 2], &vec![1, 2, 3]));
            assert_eq!(
                Ordering::Less,
                compare_vector(&vec![1, 9, 9], &vec![9, 1, 1])
            );
        }

        #[test]
        fn should_return_greater() {
            assert_eq!(Ordering::Greater, compare_vector(&vec![9], &vec![]));
            assert_eq!(Ordering::Greater, compare_vector(&vec![9], &vec![1, 2, 3]));
            assert_eq!(
                Ordering::Greater,
                compare_vector(&vec![1, 9, 2], &vec![1, 2, 2])
            );
        }
    }

    mod encode {
        struct ConditionFailWriter {
            throw_counts: Vec<u64>,
            call_count: u64,
        }

        impl ConditionFailWriter {
            fn new(throw_counts: Vec<u64>) -> ConditionFailWriter {
                ConditionFailWriter {
                    throw_counts: throw_counts,
                    call_count: 0,
                }
            }
        }

        #[cfg(not(tarpaulin_include))]
        impl std::io::Write for ConditionFailWriter {
            fn write(&mut self, bytes: &[u8]) -> std::result::Result<usize, std::io::Error> {
                self.call_count += 1;
                if self.throw_counts.contains(&self.call_count) {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
                } else {
                    Ok(bytes.len())
                }
            }

            fn write_all(&mut self, _: &[u8]) -> std::result::Result<(), std::io::Error> {
                self.call_count += 1;
                if self.throw_counts.contains(&self.call_count) {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
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
