#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod codec;
mod io;
mod prelude;

pub use codec::decode::{Decode, DecodeError};
pub use codec::encode::Encode;
pub use codec::types::{
    BENCODEX_NULL, BencodexDictionary, BencodexKey, BencodexList, BencodexValue,
};

#[cfg(feature = "json")]
pub mod json;
