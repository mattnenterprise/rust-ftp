extern crate ftp;

use std::str;
use std::io::Cursor;
use ftp::FtpStream;

fn main() {
	let mut ftp_stream = match FtpStream::connect("127.0.0.1", 21) {
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
    match ftp_stream.stor("my_random_file.txt", reader) {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    let _ = ftp_stream.quit();
}
