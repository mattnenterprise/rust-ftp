rust-ftp
================

FTP client for Rust

[![Build Status](https://travis-ci.org/mattnenterprise/rust-ftp.svg)](https://travis-ci.org/mattnenterprise/rust-ftp)
[![crates.io](http://meritbadge.herokuapp.com/ftp)](https://crates.io/crates/ftp)

[Documentation](http://mattnenterprise.github.io/rust-ftp)

### Installation

Add ftp via your `Cargo.toml`
```toml
[dependencies]
ftp = "*"
```

FTPS support is disabled by default. To enable it `secure` should be activated in `Cargo.toml`.
```toml
[dependencies]
ftp = { version = "*", features = ["secure"] }
```

### Usage
```rust
extern crate ftp;

use std::str;
use std::io::Cursor;
use ftp::FtpStream;

fn main() {
	let mut ftp_stream = match FtpStream::connect("127.0.0.1:21") {
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

    match str::from_utf8(&remote_file.into_inner()) {
        Ok(s) => print!("{}", s),
        Err(e) => panic!("Error reading file data: {}", e)
    };

    //Store a file
    let file_data = format!("Some awesome file data man!!");
    let reader: &mut Cursor<Vec<u8>> = &mut Cursor::new(file_data.into_bytes());
    match ftp_stream.put("my_random_file.txt", reader) {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    let _ = ftp_stream.quit();
}

```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
