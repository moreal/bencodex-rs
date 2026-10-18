//! Writer and error types used by [`Encode`](crate::Encode).
//!
//! With the `std` feature these are re-exports of [`std::io::Write`] and
//! [`std::io::Error`]. Without `std`, this module provides a minimal writer
//! trait and an opaque error type, so downstream crates can implement writers
//! and [`Encode`](crate::Encode) without the standard library.
//!
//! [`std::io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
//! [`std::io::Error`]: https://doc.rust-lang.org/std/io/struct.Error.html

#[cfg(feature = "std")]
pub use std::io::{Error, Write};

#[cfg(not(feature = "std"))]
mod no_std_io {
    use alloc::vec::Vec;
    use core::fmt;

    /// An opaque error returned by a writer when the `std` feature is disabled.
    ///
    /// This type does not retain a cause or an error kind. With `std` enabled,
    /// [`crate::io::Error`] is [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html) instead.
    #[derive(Debug)]
    pub struct Error {
        _private: (),
    }

    impl Error {
        /// Creates an opaque I/O error.
        pub fn new() -> Self {
            Self { _private: () }
        }
    }

    impl Default for Error {
        fn default() -> Self {
            Self::new()
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("i/o error")
        }
    }

    impl core::error::Error for Error {}

    /// A destination for encoded bytes when the `std` feature is disabled.
    ///
    /// With `std` enabled, [`crate::io::Write`] is [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) instead.
    pub trait Write {
        /// Writes all bytes in `buf` to this destination.
        ///
        /// # Errors
        ///
        /// Returns an error if the entire buffer cannot be written. The writer
        /// may have written part of `buf` before returning the error.
        fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>;
    }

    impl<W: Write + ?Sized> Write for &mut W {
        fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
            (**self).write_all(buf)
        }
    }

    impl Write for Vec<u8> {
        fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
            self.extend_from_slice(buf);
            Ok(())
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std_io::{Error, Write};
