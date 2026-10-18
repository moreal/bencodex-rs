#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, missing_debug_implementations)]

//! Encodes and decodes [Bencodex], a deterministic binary serialization format.
//!
//! [`BencodexValue`] represents values and [`BencodexKey`] restricts dictionary
//! keys to binary or text strings. [`Encode`] writes values to a caller-supplied
//! writer; [`decode_borrowed`] borrows strings from an input slice, while
//! [`Decode::decode`] returns owned data. Use [`BencodexValue::into_owned`] when
//! a borrowed value needs to outlive its input.
//!
//! ```
//! use bencodex::{BencodexValue, Encode, decode_borrowed};
//!
//! let value = BencodexValue::from("hello");
//! let mut bytes = Vec::new();
//! value.encode(&mut bytes)?;
//! assert_eq!(bytes, b"u5:hello");
//! assert_eq!(decode_borrowed(&bytes)?, value);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! - `std` (default): standard-library writers and errors.
//! - `alloc`: compatibility feature for `no_std` builds. Allocation is always
//!   required, including when all default features are disabled.
//! - `json`: conversion to and from [Bencodex JSON]; enables `std`.
//! - `json-cli`: command-line JSON conversion; enables `json`.
//! - `simd`: runtime-selected SIMD decoding; enables `std`.
//!
//! With `default-features = false`, [`io`] provides a minimal writer trait and
//! error type. The public data model and scalar codec also work without `std`.
//! See the repository's [release notes] for API changes and migration guidance.
//!
//! [Bencodex]: https://bencodex.org/
//! [Bencodex JSON]: https://github.com/planetarium/bencodex/blob/main/JSON.md
//! [release notes]: https://github.com/bencodex/bencodex-rs/blob/main/CHANGES.md

extern crate alloc;

pub mod codec;
pub mod io;
mod prelude;

pub use codec::decode::{Decode, DecodeError, decode_borrowed};
pub use codec::encode::{Encode, to_writer};
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
