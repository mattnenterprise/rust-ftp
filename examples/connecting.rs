extern crate ftp;

use std::str;
use std::io::{Cursor, Error, ErrorKind, Result};
use ftp::FtpStream;

fn test_ftp(addr: &str, user: &str, pass: &str) -> Result<()> {
    let mut ftp_stream = try!(FtpStream::connect((addr, 21)));
    try!(ftp_stream.login(user, pass));
    println!("current dir: {}", try!(ftp_stream.pwd()));

    try!(ftp_stream.cwd("test_data"));

    // An easy way to retrieve a file
    let cursor = try!(ftp_stream.simple_retr("ftpext-charter.txt"));
    let vec = cursor.into_inner();
    let text = try!(str::from_utf8(&vec).or_else(|cause|
        Err(Error::new(ErrorKind::Other, cause))
    ));
    println!("got data: {}", text);

    // Store a file
    let file_data = format!("Some awesome file data man!!");
    let mut reader = Cursor::new(file_data.into_bytes());
    try!(ftp_stream.put("my_random_file.txt", &mut reader));

    ftp_stream.quit()
}

fn main() {
    test_ftp("127.0.0.1", "Doe", "mumble").unwrap_or_else(|err|
        panic!("{}", err)
    );
    println!("test successful")
}
