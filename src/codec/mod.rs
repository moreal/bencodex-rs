//! Bencodex encoding, decoding, and value types.
//!
//! The main API is also available at the crate root; see the
//! [crate example](crate) for a round trip.

/// Scalar decoding and its error type. See [`decode::decode_borrowed`].
pub mod decode;
/// Encoding into caller-supplied writers. See [`encode::to_writer`].
pub mod encode;
/// The borrowed or owned data model. See [`types::BencodexValue`].
pub mod types;

#[cfg(feature = "simd")]
pub mod simd;
