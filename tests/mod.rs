pub mod codec;
#[cfg(all(any(feature = "json", feature = "simd"), not(miri)))]
pub mod fuzz;
#[cfg(feature = "json")]
pub mod json;
