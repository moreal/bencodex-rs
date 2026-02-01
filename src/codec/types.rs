use crate::prelude::*;
use num_bigint::BigInt;

/// The type alias of `BTreeMap<BencodexKey, BencodexValue>` to reduce code size.
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
pub type BencodexDictionary<'a> = BTreeMap<BencodexKey<'a>, BencodexValue<'a>>;
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
pub type BencodexList<'a> = Vec<BencodexValue<'a>>;

/// The constant of `BencodexValue::Null`.
///
/// ```
/// use bencodex::{ Encode, BENCODEX_NULL };
///
/// let mut buf = vec![];
/// BENCODEX_NULL.encode(&mut buf);
/// assert_eq!(buf, b"n")
/// ```
pub const BENCODEX_NULL: BencodexValue<'static> = BencodexValue::Null;

#[derive(PartialEq, Debug, Clone)]
pub enum BencodexValue<'a> {
    Binary(Cow<'a, [u8]>),
    Text(Cow<'a, str>),
    Boolean(bool),
    Number(BigInt),
    List(BencodexList<'a>),
    Dictionary(BencodexDictionary<'a>),
    Null,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Clone, Ord)]
pub enum BencodexKey<'a> {
    Binary(Cow<'a, [u8]>),
    Text(Cow<'a, str>),
}

impl<'a> BencodexValue<'a> {
    /// Convert a borrowed `BencodexValue` into a fully owned one with `'static` lifetime.
    pub fn into_owned(self) -> BencodexValue<'static> {
        match self {
            BencodexValue::Binary(cow) => BencodexValue::Binary(Cow::Owned(cow.into_owned())),
            BencodexValue::Text(cow) => BencodexValue::Text(Cow::Owned(cow.into_owned())),
            BencodexValue::Boolean(b) => BencodexValue::Boolean(b),
            BencodexValue::Number(n) => BencodexValue::Number(n),
            BencodexValue::List(list) => {
                BencodexValue::List(list.into_iter().map(|v| v.into_owned()).collect())
            }
            BencodexValue::Dictionary(dict) => BencodexValue::Dictionary(
                dict.into_iter()
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect(),
            ),
            BencodexValue::Null => BencodexValue::Null,
        }
    }
}

impl<'a> BencodexKey<'a> {
    /// Convert a borrowed `BencodexKey` into a fully owned one with `'static` lifetime.
    pub fn into_owned(self) -> BencodexKey<'static> {
        match self {
            BencodexKey::Binary(cow) => BencodexKey::Binary(Cow::Owned(cow.into_owned())),
            BencodexKey::Text(cow) => BencodexKey::Text(Cow::Owned(cow.into_owned())),
        }
    }
}

impl<'a> From<&'a str> for BencodexKey<'a> {
    fn from(val: &'a str) -> Self {
        BencodexKey::Text(Cow::Borrowed(val))
    }
}

impl From<String> for BencodexKey<'_> {
    fn from(val: String) -> Self {
        BencodexKey::Text(Cow::Owned(val))
    }
}

impl From<Vec<u8>> for BencodexKey<'_> {
    fn from(val: Vec<u8>) -> Self {
        BencodexKey::Binary(Cow::Owned(val))
    }
}

impl<'a> From<&'a [u8]> for BencodexKey<'a> {
    fn from(val: &'a [u8]) -> Self {
        BencodexKey::Binary(Cow::Borrowed(val))
    }
}

impl<'a> From<&'a [u8]> for BencodexValue<'a> {
    fn from(val: &'a [u8]) -> Self {
        BencodexValue::Binary(Cow::Borrowed(val))
    }
}

impl From<Vec<u8>> for BencodexValue<'_> {
    fn from(val: Vec<u8>) -> Self {
        BencodexValue::Binary(Cow::Owned(val))
    }
}

impl<'a> From<&'a str> for BencodexValue<'a> {
    fn from(val: &'a str) -> Self {
        BencodexValue::Text(Cow::Borrowed(val))
    }
}

impl From<String> for BencodexValue<'_> {
    fn from(val: String) -> Self {
        BencodexValue::Text(Cow::Owned(val))
    }
}

macro_rules! bencodex_value_number_impl {
    ($x:tt) => {
        impl From<$x> for BencodexValue<'_> {
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

impl From<bool> for BencodexValue<'_> {
    fn from(val: bool) -> Self {
        BencodexValue::Boolean(val)
    }
}

impl<'a, T> From<Vec<T>> for BencodexValue<'a>
where
    T: Into<BencodexValue<'a>>,
{
    fn from(val: Vec<T>) -> Self {
        let mut vec = Vec::new();
        for v in val {
            vec.push(v.into());
        }

        BencodexValue::List(vec)
    }
}

impl<'a, T, U> From<BTreeMap<T, U>> for BencodexValue<'a>
where
    T: Into<BencodexKey<'a>>,
    U: Into<BencodexValue<'a>>,
{
    fn from(val: BTreeMap<T, U>) -> Self {
        let mut map = BTreeMap::<BencodexKey<'a>, BencodexValue<'a>>::new();
        for (key, value) in val {
            map.insert(key.into(), value.into());
        }

        BencodexValue::Dictionary(map)
    }
}

#[cfg(test)]
mod tests {
    mod into {
        use crate::prelude::*;
        use alloc::vec;

        use super::super::{BencodexKey, BencodexValue};

        #[test]
        fn text() {
            let s: &str = "value";
            let value: BencodexKey = s.into();
            assert_eq!(value, BencodexKey::Text(Cow::Borrowed("value")));

            let s: String = "value".to_string();
            let value: BencodexKey = s.into();
            assert_eq!(value, BencodexKey::Text(Cow::Owned("value".to_string())));

            let s: &str = "value";
            let value: BencodexValue = s.into();
            assert_eq!(value, BencodexValue::Text(Cow::Borrowed("value")));

            let s: String = "value".to_string();
            let value: BencodexValue = s.into();
            assert_eq!(value, BencodexValue::Text(Cow::Owned("value".to_string())));
        }

        #[test]
        fn binary() {
            let b: &[u8] = &[0, 1, 2, 3];
            let value: BencodexKey = b.into();
            assert_eq!(value, BencodexKey::Binary(Cow::Borrowed(&[0, 1, 2, 3])));

            let b: Vec<u8> = vec![0, 1, 2, 3];
            let value: BencodexKey = b.into();
            assert_eq!(value, BencodexKey::Binary(Cow::Owned(vec![0, 1, 2, 3])));

            let b: &[u8] = &[0, 1, 2, 3];
            let value: BencodexValue = b.into();
            assert_eq!(value, BencodexValue::Binary(Cow::Borrowed(&[0, 1, 2, 3])));

            let b: Vec<u8> = vec![0, 1, 2, 3];
            let value: BencodexValue = b.into();
            assert_eq!(value, BencodexValue::Binary(Cow::Owned(vec![0, 1, 2, 3])));
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

            let expected = BencodexValue::Dictionary(BTreeMap::from_iter([(
                BencodexKey::Text(Cow::Owned("foo".to_string())),
                BencodexValue::Binary(Cow::Borrowed(b"bar".as_slice())),
            )]));

            assert_eq!(actual, expected);
        }
    }
}
