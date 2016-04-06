#![crate_name = "ftp"]
#![crate_type = "lib"]

//! ftp is an FTP client written in Rust.
//!
//! ### Usage
//!
//! Here is a basic usage example:
//!
//! ```rust
//! use ftp::FtpStream;
//! let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap_or_else(|err|
//!     panic!("{}", err)
//! );
//! let _ = ftp_stream.quit();
//! ```
//!

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
#[cfg(feature = "secure")]
extern crate openssl;

mod ftp;
mod data_stream;
pub mod types;
pub mod status;

pub use self::ftp::FtpStream;
