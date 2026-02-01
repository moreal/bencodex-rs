#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod codec;
mod io;
mod prelude;

pub use codec::decode::{Decode, DecodeError, decode_borrowed};
pub use codec::encode::Encode;
pub use codec::types::{
    BENCODEX_NULL, BencodexDictionary, BencodexKey, BencodexList, BencodexValue,
};

#[cfg(feature = "json")]
pub mod json;

/// SIMD-accelerated decoding module.
///
/// Provides `decode_simd()` function for faster Bencodex decoding using
/// SIMD instructions on supported platforms (x86_64 SSE4.2/AVX2, AArch64 NEON).
#[cfg(feature = "simd")]
pub mod simd {
    pub use crate::codec::simd::decode_simd;
}
