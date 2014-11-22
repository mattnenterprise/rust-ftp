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
use std::slice::bytes::MutableByteVector;
use ftp::FTPStream;

fn main() {
	let mut ftp_stream = match FTPStream::connect("ftp.ietf.org", 21) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    match ftp_stream.login("anonymous", "somedude@yahoo.com") {
    	Ok(_) => (),
    	Err(e) => panic!("{}", e)
    }

    match ftp_stream.change_dir("ietf/ftpext/") {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    let mut data_stream = match ftp_stream.retr("ftpext-charter.txt") {
        Ok(data_stream) => data_stream,
        Err(e) => panic!("{}", e)
    };

    let mut buf = [0, ..100];
    let mut still_reading = true;
    while still_reading {
        buf.set_memory(0);
        match data_stream.read(&mut buf) {
            Ok(nread) => {
                if nread == 0 {
                    still_reading = false;
                } else {
                    match str::from_utf8(&buf) {
                        Some(s) => print!("{}", s),
                        None => panic!("Failure")
                    };
                }
            }
            Err(_) => still_reading = false,
        }
    }

    drop(data_stream);

    let _ = ftp_stream.quit();
}
```

### License

MIT