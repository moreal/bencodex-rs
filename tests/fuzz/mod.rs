use bencodex::{BencodexDictionary, BencodexKey, BencodexList, BencodexValue};
use num_bigint::BigInt;
use quickcheck::{Arbitrary, Gen};

#[cfg(feature = "json")]
pub mod json_encode;

#[derive(Clone, Debug)]
pub struct ArbitraryBencodexKey(pub BencodexKey);

#[derive(Clone, Debug)]
pub struct ArbitraryBencodexValue(pub BencodexValue);

impl Arbitrary for ArbitraryBencodexKey {
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            ArbitraryBencodexKey(BencodexKey::Binary(Vec::arbitrary(g)))
        } else {
            ArbitraryBencodexKey(BencodexKey::Text(String::arbitrary(g)))
        }
    }
}

fn arbitrary_bigint(g: &mut Gen) -> BigInt {
    let bytes: Vec<u8> = Vec::arbitrary(g);
    if bytes.is_empty() {
        BigInt::from(0)
    } else {
        BigInt::from_signed_bytes_be(&bytes)
    }
}

fn leaf_value(g: &mut Gen) -> BencodexValue {
    match u8::arbitrary(g) % 5 {
        0 => BencodexValue::Null,
        1 => BencodexValue::Boolean(bool::arbitrary(g)),
        2 => BencodexValue::Number(arbitrary_bigint(g)),
        3 => BencodexValue::Binary(Vec::arbitrary(g)),
        _ => BencodexValue::Text(String::arbitrary(g)),
    }
}

impl Arbitrary for ArbitraryBencodexValue {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = g.size();
        if size <= 1 {
            return ArbitraryBencodexValue(leaf_value(g));
        }

        let value = match u8::arbitrary(g) % 7 {
            0 => BencodexValue::Null,
            1 => BencodexValue::Boolean(bool::arbitrary(g)),
            2 => BencodexValue::Number(arbitrary_bigint(g)),
            3 => BencodexValue::Binary(Vec::arbitrary(g)),
            4 => BencodexValue::Text(String::arbitrary(g)),
            5 => {
                let len = usize::arbitrary(g) % size;
                let smaller_size = std::cmp::max(1, size / 2);
                let mut smaller = Gen::new(smaller_size);
                let list: BencodexList = (0..len)
                    .map(|_| ArbitraryBencodexValue::arbitrary(&mut smaller).0)
                    .collect();
                BencodexValue::List(list)
            }
            _ => {
                let len = usize::arbitrary(g) % size;
                let smaller_size = std::cmp::max(1, size / 2);
                let mut smaller = Gen::new(smaller_size);
                let dict: BencodexDictionary = (0..len)
                    .map(|_| {
                        (
                            ArbitraryBencodexKey::arbitrary(&mut smaller).0,
                            ArbitraryBencodexValue::arbitrary(&mut smaller).0,
                        )
                    })
                    .collect();
                BencodexValue::Dictionary(dict)
            }
        };
        ArbitraryBencodexValue(value)
    }
}
