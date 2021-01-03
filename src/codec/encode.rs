use super::types::*;
use itertools::Itertools;
use num_bigint::BigInt;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::io;
use std::result::Result;

pub trait Encode {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error>;
}

impl Encode for Vec<u8> {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        match write!(writer, "{}:", self.len()) {
            Ok(()) => match writer.write(&self) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

impl Encode for i64 {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        write!(writer, "i{}e", self)
    }
}

impl Encode for String {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        let bytes = self.into_bytes();
        match write!(writer, "u{}:", bytes.len()) {
            Ok(()) => match writer.write(&bytes) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

impl Encode for bool {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        match writer.write(match self {
            true => &[b't'],
            false => &[b'f'],
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Encode for BigInt {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        if let Err(e) = writer.write(&[b'i']) {
            return Err(e);
        }

        if let Err(e) = writer.write(&self.to_str_radix(10).into_bytes()) {
            return Err(e);
        }

        if let Err(e) = writer.write(&[b'e']) {
            return Err(e);
        }

        Ok(())
    }
}

impl Encode for Vec<BencodexValue> {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        if let Err(e) = writer.write(&[b'l']) {
            return Err(e);
        }

        for el in self {
            if let Err(e) = el.encode(writer) {
                return Err(e);
            }
        }

        if let Err(e) = writer.write(&[b'e']) {
            return Err(e);
        }

        Ok(())
    }
}

impl Encode for () {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        match writer.write(&[b'n']) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn encode_key(key: &BencodexKey) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    let (prefix, bytes) = match key {
        BencodexKey::Text(s) => (Some(vec![b'u']), s.to_owned().into_bytes()),
        BencodexKey::Binary(b) => (None as Option<Vec<u8>>, b.clone()),
    };
    match prefix {
        Some(p) => buf.extend(p),
        _ => (),
    };

    buf.extend(bytes.len().to_string().into_bytes());
    buf.push(b':');
    buf.extend(bytes);
    buf
}

impl Encode for BencodexValue {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        // FIXME: rewrite more beautiful.
        match match self {
            BencodexValue::Binary(x) => x.encode(writer),
            BencodexValue::Text(x) => x.encode(writer),
            BencodexValue::Dictionary(x) => x.encode(writer),
            BencodexValue::List(x) => x.encode(writer),
            BencodexValue::Boolean(x) => x.encode(writer),
            BencodexValue::Null(x) => x.encode(writer),
            BencodexValue::Number(x) => x.encode(writer),
        } {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn compare_vector<T: Ord>(xs: &Vec<T>, ys: &Vec<T>) -> Ordering {
    for (x, y) in xs.iter().zip(ys) {
        match x.cmp(&y) {
            Ordering::Equal => continue,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
        };
    }

    xs.len().cmp(&ys.len())
}

impl Encode for BTreeMap<BencodexKey, BencodexValue> {
    fn encode(self, writer: &mut dyn io::Write) -> Result<(), std::io::Error> {
        let pairs = self
            .into_iter()
            .map(|(key, value)| {
                let key_bytes = encode_key(&key);
                (key, key_bytes, value)
            })
            .sorted_by(|(x_key, x_key_bytes, _), (y_key, y_key_bytes, _)| {
                match x_key {
                    BencodexKey::Text(_) => return Ordering::Greater,
                    _ => (),
                };

                match y_key {
                    BencodexKey::Text(_) => return Ordering::Less,
                    _ => (),
                };

                compare_vector(&x_key_bytes, &y_key_bytes)
            });

        if let Err(e) = writer.write(&[b'd']) {
            return Err(e);
        }

        for (_, key_bytes, value) in pairs {
            if let Err(e) = writer.write(&key_bytes) {
                return Err(e);
            }

            if let Err(e) = value.encode(writer) {
                return Err(e);
            }
        }

        if let Err(e) = writer.write(&[b'e']) {
            return Err(e);
        }

        Ok(())
    }
}