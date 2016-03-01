#[cfg(test)]

extern crate ftp;

use std::io::Cursor;
use ftp::FtpStream;

#[test]
fn test_ftp() {
    let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();
    ftp_stream.login("anonymous", "rust-ftp@github.com").unwrap_or_else(|e| panic!("{}", e));

    ftp_stream.mkdir("test_folder").unwrap();
    ftp_stream.cwd("test_folder").unwrap();
    assert_eq!(ftp_stream.pwd().unwrap(), "/test_folder");

    // Store a file
    let file_data = format!("Some awesome file data man!!\n");
    let mut reader = Cursor::new(file_data.into_bytes());
    assert!(ftp_stream.put("test_file.txt", &mut reader).is_ok());

    assert!(ftp_stream.quit().is_ok());
}
