pub mod codec;
#[cfg(any(feature = "json", feature = "simd"))]
pub mod fuzz;
#[cfg(feature = "json")]
pub mod json;
