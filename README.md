rust-ftp
================
FTP client for Rust

This client isn't really finished yet. It still requires a lot of work.


[![Build Status](https://travis-ci.org/mattnenterprise/rust-ftp.svg)](https://travis-ci.org/mattnenterprise/rust-ftp)

### Installation

Add ftp via your `Cargo.toml`
```toml
[dependencies.ftp]
git = "https://github.com/mattnenterprise/rust-ftp"
```

### Usage
```rs
extern crate ftp;

use std::str;
use std::io::{MemReader};
use ftp::FTPStream;

fn main() {
    let mut ftp_stream = match FTPStream::connect("127.0.0.1", 21) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    match ftp_stream.login("username", "password") {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    match ftp_stream.current_dir() {
        Ok(dir) => println!("{}", dir),
        Err(e) => panic!("{}", e)
    }

    match ftp_stream.change_dir("test_data") {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    //An easy way to retreive a file
    let remote_file = match ftp_stream.simple_retr("ftpext-charter.txt") {
        Ok(file) => file,
        Err(e) => panic!("{}", e)
    };

    match str::from_utf8(remote_file.into_inner().as_slice()) {
        Some(s) => print!("{}", s),
        None => panic!("Error reading file data")
    };

    //Store a file
    let file_data = format!("Some awesome file data man!!");
    let reader = &mut MemReader::new(file_data.into_bytes());
    match ftp_stream.stor("my_random_file.txt", reader) {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    let _ = ftp_stream.quit();
}
```

### License

MIT