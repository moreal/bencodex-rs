//! Architecture-specific SIMD backends.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

pub mod fallback;

/// Trait defining SIMD backend operations.
///
/// Each architecture (SSE4.2, AVX2, NEON) implements this trait to provide
/// consistent vector operations for structural character detection.
pub trait SimdBackend {
    /// Width of vector lane in bytes (16 for SSE/NEON, 32 for AVX2)
    const LANE_WIDTH: usize;

    /// Vector type for this backend
    type Vector: Copy;

    /// Load unaligned bytes from memory into a vector
    ///
    /// # Safety
    /// - `ptr` must point to at least `LANE_WIDTH` readable bytes
    unsafe fn load_unaligned(ptr: *const u8) -> Self::Vector;

    /// Compare each byte in vector `a` against byte `b`
    /// Returns a mask where matching positions are 0xFF, others are 0x00
    ///
    /// # Safety
    /// - Requires the appropriate SIMD feature to be available
    unsafe fn cmpeq_epi8(a: Self::Vector, b: u8) -> Self::Vector;

    /// Bitwise OR of two vectors
    ///
    /// # Safety
    /// - Requires the appropriate SIMD feature to be available
    unsafe fn or(a: Self::Vector, b: Self::Vector) -> Self::Vector;

    /// Extract most significant bit of each byte into a bitmask
    /// Bit N is set if byte N has its MSB set (0x80 or higher)
    ///
    /// # Safety
    /// - Requires the appropriate SIMD feature to be available
    unsafe fn movemask_epi8(a: Self::Vector) -> u32;
}
