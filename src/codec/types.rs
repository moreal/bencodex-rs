use num_bigint::BigInt;

#[cfg(feature = "std")]
use std::{collections::BTreeMap, fmt::Debug};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use core::fmt::Debug;

/// The type alias of `BTreepMap<BencodexKey, BencodexValue>` to reduce code size.
///
/// ```
/// use bencodex::{ Encode, BencodexDictionary };
///
/// let mut dict = BencodexDictionary::new();
/// dict.insert("foo".into(), "bar".into());
///
/// let mut buf = vec![];
/// dict.encode(&mut buf);
/// assert_eq!(buf, b"du3:foou3:bare")
/// ```
pub type BencodexDictionary = BTreeMap<BencodexKey, BencodexValue>;
/// The type alias of `Vec<BencodexValue>` to reduce code size.
///
/// ```
/// use bencodex::{ Encode, BencodexList };
///
/// let mut list = BencodexList::new();
/// list.push("foo".to_string().into());
/// list.push("bar".to_string().into());
///
/// let mut buf = vec![];
/// list.encode(&mut buf);
/// assert_eq!(buf, b"lu3:foou3:bare")
/// ```
pub type BencodexList = Vec<BencodexValue>;

/// The constant of `BencodexValue::Null`.
///
/// ```
/// use bencodex::{ Encode, BENCODEX_NULL };
///
/// let mut buf = vec![];
/// BENCODEX_NULL.encode(&mut buf);
/// assert_eq!(buf, b"n")
/// ```
pub const BENCODEX_NULL: BencodexValue = BencodexValue::Null;

#[derive(PartialEq, Debug, Clone)]
pub enum BencodexValue {
    Binary(Vec<u8>),
    Text(String),
    Boolean(bool),
    Number(BigInt),
    List(BencodexList),
    Dictionary(BencodexDictionary),
    Null,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Clone, Ord)]
pub enum BencodexKey {
    Binary(Vec<u8>),
    Text(String),
}

impl From<&str> for BencodexKey {
    fn from(val: &str) -> Self {
        BencodexKey::Text(val.to_string())
    }
}

impl From<String> for BencodexKey {
    fn from(val: String) -> Self {
        BencodexKey::Text(val)
    }
}

impl From<&String> for BencodexKey {
    fn from(val: &String) -> Self {
        BencodexKey::Text(val.clone())
    }
}

impl From<Vec<u8>> for BencodexKey {
    fn from(val: Vec<u8>) -> Self {
        BencodexKey::Binary(val)
    }
}

impl From<&Vec<u8>> for BencodexKey {
    fn from(val: &Vec<u8>) -> Self {
        BencodexKey::Binary(val.clone())
    }
}

impl From<&[u8]> for BencodexKey {
    fn from(val: &[u8]) -> Self {
        BencodexKey::Binary(val.to_vec())
    }
}

impl From<&[u8]> for BencodexValue {
    fn from(val: &[u8]) -> Self {
        BencodexValue::Binary(val.to_vec())
    }
}

impl From<Vec<u8>> for BencodexValue {
    fn from(val: Vec<u8>) -> Self {
        BencodexValue::Binary(val)
    }
}

impl From<&str> for BencodexValue {
    fn from(val: &str) -> Self {
        BencodexValue::Text(val.to_string())
    }
}

impl From<String> for BencodexValue {
    fn from(val: String) -> Self {
        BencodexValue::Text(val)
    }
}

macro_rules! bencodex_value_number_impl {
    ($x:tt) => {
        impl From<$x> for BencodexValue {
            fn from(val: $x) -> Self {
                BencodexValue::Number(val.into())
            }
        }
    };
}

bencodex_value_number_impl!(u16);
bencodex_value_number_impl!(u32);
bencodex_value_number_impl!(u64);
bencodex_value_number_impl!(i8);
bencodex_value_number_impl!(i16);
bencodex_value_number_impl!(i32);
bencodex_value_number_impl!(i64);

impl From<bool> for BencodexValue {
    fn from(val: bool) -> Self {
        BencodexValue::Boolean(val)
    }
}

impl<T> From<Vec<T>> for BencodexValue
where
    T: Into<BencodexValue>,
{
    fn from(val: Vec<T>) -> Self {
        let mut vec = Vec::new();
        for v in val {
            vec.push(v.into());
        }

        BencodexValue::List(vec)
    }
}

impl<T, U> From<BTreeMap<T, U>> for BencodexValue
where
    T: Into<BencodexKey>,
    U: Into<BencodexValue>,
{
    fn from(val: BTreeMap<T, U>) -> Self {
        let mut map = BTreeMap::<BencodexKey, BencodexValue>::new();
        for (key, value) in val {
            map.insert(key.into(), value.into());
        }

        BencodexValue::Dictionary(map)
    }
}

#[cfg(test)]
mod tests {
    mod into {
        #[cfg(feature = "std")]
        use std::array::IntoIter;
        #[cfg(feature = "std")]
        use std::{collections::BTreeMap, iter::FromIterator};

        #[cfg(not(feature = "std"))]
        use alloc::collections::BTreeMap;
        #[cfg(not(feature = "std"))]
        use core::array::IntoIter;
        #[cfg(not(feature = "std"))]
        use core::iter::FromIterator;

        use super::super::{BencodexKey, BencodexValue};

        #[test]
        fn text() {
            let s: &str = "value";
            let value: BencodexKey = s.into();
            assert_eq!(value, BencodexKey::Text("value".to_string()));

            let s: String = "value".to_string();
            let value: BencodexKey = s.into();
            assert_eq!(value, BencodexKey::Text("value".to_string()));

            let s: &str = "value";
            let value: BencodexValue = s.into();
            assert_eq!(value, BencodexValue::Text("value".to_string()));

            let s: String = "value".to_string();
            let value: BencodexValue = s.into();
            assert_eq!(value, BencodexValue::Text("value".to_string()));
        }

        #[test]
        fn binary() {
            let b: &[u8] = &[0, 1, 2, 3];
            let value: BencodexKey = b.into();
            assert_eq!(value, BencodexKey::Binary(vec![0, 1, 2, 3]));

            let b: Vec<u8> = vec![0, 1, 2, 3];
            let value: BencodexKey = b.into();
            assert_eq!(value, BencodexKey::Binary(vec![0, 1, 2, 3]));

            let b: &[u8] = &[0, 1, 2, 3];
            let value: BencodexValue = b.into();
            assert_eq!(value, BencodexValue::Binary(vec![0, 1, 2, 3]));

            let b: Vec<u8> = vec![0, 1, 2, 3];
            let value: BencodexValue = b.into();
            assert_eq!(value, BencodexValue::Binary(vec![0, 1, 2, 3]));
        }

        #[test]
        fn number() {
            let n: u16 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: u32 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: u64 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: i8 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: i16 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: i32 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));

            let n: i64 = 0;
            let value: BencodexValue = n.into();
            assert_eq!(value, BencodexValue::Number(0.into()));
        }

        #[test]
        fn boolean() {
            let value: BencodexValue = true.into();
            assert_eq!(value, BencodexValue::Boolean(true));

            let value: BencodexValue = false.into();
            assert_eq!(value, BencodexValue::Boolean(false));
        }

        #[test]
        fn null() {
            let value: BencodexValue = BencodexValue::Null;
            assert_eq!(value, BencodexValue::Null);
        }

        #[test]
        fn list() {
            let l = vec!["A", "B", "C", "D"];
            let value: BencodexValue = l.into();
            assert_eq!(
                value,
                BencodexValue::List(vec!["A".into(), "B".into(), "C".into(), "D".into()])
            );

            let l = vec![0, 1, 2, 3];
            let value: BencodexValue = l.into();
            assert_eq!(
                value,
                BencodexValue::List(vec![0.into(), 1.into(), 2.into(), 3.into()])
            );

            let l = vec![
                BencodexValue::Null,
                BencodexValue::Null,
                BencodexValue::Null,
            ];
            let value: BencodexValue = l.into();
            assert_eq!(
                value,
                BencodexValue::List(vec![
                    BencodexValue::Null,
                    BencodexValue::Null,
                    BencodexValue::Null
                ])
            );

            let l: Vec<Vec<u8>> = vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7]];
            let value: BencodexValue = l.into();
            assert_eq!(
                value,
                BencodexValue::List(vec![vec![0u8, 1, 2, 3].into(), vec![4u8, 5, 6, 7].into(),])
            );
        }

        #[test]
        fn dictionary() {
            let mut map = BTreeMap::<String, &[u8]>::new();
            map.insert("foo".to_string(), b"bar");
            let actual: BencodexValue = map.into();

            let expected = BencodexValue::Dictionary(BTreeMap::from_iter(IntoIter::new([(
                BencodexKey::Text("foo".to_string()),
                BencodexValue::Binary(vec![b'b', b'a', b'r']),
            )])));

            assert_eq!(actual, expected);
        }
    }
}
