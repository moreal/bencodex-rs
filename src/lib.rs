pub mod codec;

pub use codec::decode::{Decode, DecodeError};
pub use codec::encode::Encode;
pub use codec::types::{
    BencodexDictionary, BencodexKey, BencodexList, BencodexValue, BENCODEX_NULL,
};

#[cfg(feature = "json")]
pub mod json;
