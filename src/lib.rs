pub mod codec;

pub use codec::decode::{Decode, DecodeError};
pub use codec::encode::Encode;
pub use codec::types::{
    BENCODEX_NULL, BencodexDictionary, BencodexKey, BencodexList, BencodexValue,
};

#[cfg(feature = "json")]
pub mod json;
