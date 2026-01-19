#[cfg(feature = "std")]
pub use std::io::{Error, Write};

#[cfg(not(feature = "std"))]
mod no_std_io {
    use alloc::vec::Vec;
    use core::fmt;

    #[derive(Debug)]
    pub struct Error {
        _private: (),
    }

    impl Error {
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
            write!(f, "I/O error")
        }
    }

    impl core::error::Error for Error {}

    pub trait Write {
        fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>;
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
