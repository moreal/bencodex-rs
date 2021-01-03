use super::types::*;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::result::Result;
use std::str;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum DecodeErrorReason {
    InvalidBencodexValue,
    UnexpectedToken { token: u8, point: usize },
}

#[derive(Debug)]
pub struct DecodeError {
    pub reason: DecodeErrorReason,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DecodableError (reason: {:?})", self.reason)
    }
}

impl Error for DecodeError {}

pub trait Decode {
    fn decode(self) -> Result<BencodexValue, DecodeError>;
}

fn decode_impl(vector: &Vec<u8>, start: usize) -> Result<(BencodexValue, usize), DecodeError> {
    if start >= vector.len() {
        return Err(DecodeError {
            reason: DecodeErrorReason::InvalidBencodexValue,
        });
    }

    match vector[start] {
        b'd' => decode_dict_impl(vector, start),
        b'l' => decode_list_impl(vector, start),
        b'u' => decode_unicode_string_impl(vector, start),
        b'i' => decode_number_impl(vector, start),
        b'0'..=b'9' => decode_byte_string_impl(vector, start),
        b't' => Ok((BencodexValue::Boolean(true), 1)),
        b'f' => Ok((BencodexValue::Boolean(false), 1)),
        b'n' => Ok((BencodexValue::Null(()), 1)),
        _ => Err(DecodeError {
            reason: DecodeErrorReason::UnexpectedToken {
                token: vector[start],
                point: start,
            },
        }),
    }
}

// start must be on 'd'
fn decode_dict_impl(vector: &Vec<u8>, start: usize) -> Result<(BencodexValue, usize), DecodeError> {
    let mut tsize: usize = 1;
    let mut map = BTreeMap::new();
    while vector[start + tsize] != b'e' {
        if start + tsize >= vector.len() {
            return Err(DecodeError {
                reason: DecodeErrorReason::InvalidBencodexValue,
            });
        }

        let index = start + tsize;
        let (value, size) = match decode_impl(vector, index) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        tsize += size;
        let key = match value {
            BencodexValue::Text(s) => BencodexKey::Text(s),
            BencodexValue::Binary(b) => BencodexKey::Binary(b),
            _ => {
                return Err(DecodeError {
                    reason: DecodeErrorReason::InvalidBencodexValue,
                })
            }
        };
        let index = start + tsize;
        let (value, size) = match decode_impl(vector, index) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        tsize += size;
        match map.insert(key, value) {
            None => (),
            Some(_) => todo!(),
        };
    }
    Ok((BencodexValue::Dictionary(map), tsize + 1))
}

// start must be on 'l'
fn decode_list_impl(vector: &Vec<u8>, start: usize) -> Result<(BencodexValue, usize), DecodeError> {
    let mut tsize: usize = 1;
    let mut list = Vec::new();
    while start + tsize < vector.len() && vector[start + tsize] != b'e' {
        let index = start + tsize;
        let (value, size) = match decode_impl(vector, index) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        tsize += size;
        list.push(value);
    }

    Ok((BencodexValue::List(list), tsize + 1))
}

fn decode_byte_string_impl(
    vector: &Vec<u8>,
    start: usize,
) -> Result<(BencodexValue, usize), DecodeError> {
    let mut tsize: usize = 0;
    let (length, size) = match read_number(&vector[start + tsize..]) {
        None => {
            return Err(DecodeError {
                reason: DecodeErrorReason::InvalidBencodexValue,
            })
        }
        Some(v) => v,
    };
    tsize += size;

    if vector[start + tsize] != b':' {
        return Err(DecodeError {
            reason: DecodeErrorReason::UnexpectedToken {
                token: vector[start + tsize],
                point: start + tsize,
            },
        });
    };
    tsize += 1;
    let length_size = length.to_usize().unwrap();
    Ok((
        BencodexValue::Binary(vector[start + tsize..start + tsize + length_size].to_vec()),
        tsize + length_size,
    ))
}

// start must be on 'u'
fn decode_unicode_string_impl(
    vector: &Vec<u8>,
    start: usize,
) -> Result<(BencodexValue, usize), DecodeError> {
    let mut tsize: usize = 1;
    let (length, size) = match read_number(&vector[start + tsize..]) {
        None => {
            return Err(DecodeError {
                reason: DecodeErrorReason::InvalidBencodexValue,
            })
        }
        Some(v) => v,
    };
    tsize += size;

    if vector[start + tsize] != b':' {
        return Err(DecodeError {
            reason: DecodeErrorReason::UnexpectedToken {
                token: vector[start + tsize],
                point: start + tsize,
            },
        });
    };

    tsize += 1;
    let length_size = length.to_usize().unwrap();
    let text = match str::from_utf8(&vector[start + tsize..start + tsize + length_size]) {
        Ok(v) => v,
        Err(e) => {
            return Err(DecodeError {
                reason: DecodeErrorReason::InvalidBencodexValue,
            })
        }
    };
    tsize += length_size;
    Ok((BencodexValue::Text(text.to_string()), tsize))
}

// start must be on 'i'
fn decode_number_impl(
    vector: &Vec<u8>,
    start: usize,
) -> Result<(BencodexValue, usize), DecodeError> {
    let mut tsize: usize = 1;
    let (number, size) = match read_number(&vector[start + tsize..]) {
        None => {
            return Err(DecodeError {
                reason: DecodeErrorReason::InvalidBencodexValue,
            })
        }
        Some(v) => v,
    };
    tsize += size;

    if vector[start + tsize] != b'e' {
        Err(DecodeError {
            reason: DecodeErrorReason::UnexpectedToken {
                token: vector[start + tsize],
                point: start + tsize,
            },
        })
    } else {
        tsize += 1;
        Ok((BencodexValue::Number(number), tsize))
    }
}

fn read_number(s: &[u8]) -> Option<(BigInt, usize)> {
    let mut size: usize = 0;
    loop {
        size += 1;
        match s[size] {
            b'0'..=b'9' => continue,
            _ => break,
        };
    }

    if size == 0 {
        None
    } else {
        Some((
            BigInt::from_str(&String::from_utf8(s[..size].to_vec()).unwrap()).unwrap(),
            size,
        ))
    }
}

impl Decode for Vec<u8> {
    fn decode(self) -> Result<BencodexValue, DecodeError> {
        match decode_impl(&self, 0) {
            Ok(v) => Ok(v.0),
            Err(e) => Err(e),
        }
    }
}