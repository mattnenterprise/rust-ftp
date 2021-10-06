#[cfg(test)]
extern crate ftp;

#[cfg(feature = "integration-test")]
use ftp::FtpStream;
#[cfg(feature = "integration-test")]
use std::io::Cursor;

#[test]
#[cfg(feature = "integration-test")]
fn get_welcome_msg() {
    let ftp_stream = setup_stream();
    assert_eq!(ftp_stream.get_welcome_msg(), Some("220 (vsFTPd 3.0.3)"));
}

#[test]
#[cfg(feature = "integration-test")]
fn noop() {
    let mut ftp_stream = setup_stream();
    assert!(ftp_stream.noop().is_ok());
}

#[test]
#[cfg(feature = "integration-test")]
fn test_ftp() {
    let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();

    println!("Welcome message: {:?}", ftp_stream.get_welcome_msg());

    let _ = ftp_stream.login("Doe", "mumble").unwrap();

    ftp_stream.mkdir("test_dir").unwrap();
    ftp_stream.cwd("test_dir").unwrap();
    assert!(ftp_stream.pwd().unwrap().ends_with("/test_dir"));

    // store a file
    let file_data = "test data\n";
    let mut reader = Cursor::new(file_data.as_bytes());
    assert!(ftp_stream.put("test_file.txt", &mut reader).is_ok());

    // retrieve file
    assert!(ftp_stream
        .simple_retr("test_file.txt")
        .map(|bytes| assert_eq!(bytes.into_inner(), file_data.as_bytes()))
        .is_ok());

    // remove file
    assert!(ftp_stream.rm("test_file.txt").is_ok());

    // cleanup: go up, remove folder, and quit
    assert!(ftp_stream
        .cdup()
        .and_then(|_| ftp_stream.rmdir("test_dir"))
        .and_then(|_| ftp_stream.quit())
        .is_ok());
}

#[cfg(feature = "integration-test")]
fn setup_stream() -> FtpStream {
    let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();
    assert!(ftp_stream.login("Doe", "mumble").is_ok());
    ftp_stream
}
