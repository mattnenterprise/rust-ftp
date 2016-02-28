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

use std::io::{Error, ErrorKind, Read, Result, BufRead, BufReader, BufWriter, Cursor, Write, copy};
use std::net::TcpStream;
use std::string::String;
use std::net::ToSocketAddrs;

/// Stream to interface with the FTP server. This interface is only for the command stream.
#[derive(Debug)]
pub struct FtpStream {
    reader: BufReader<TcpStream>
}

impl FtpStream {
    /// Creates an FTP Stream.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<FtpStream> {
        let reader = BufReader::new(try!(TcpStream::connect(addr)));
        let mut ftp_stream = FtpStream {
            reader: reader
        };

        try!(ftp_stream.read_response(220));
        Ok(ftp_stream)
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        let stream = self.reader.get_mut();
        return stream.write_fmt(format_args!("{}", s));
    }

    /// Log in to the FTP server.
    pub fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let user_command = format!("USER {}\r\n", user);
        try!(self.write_str(&user_command));

        self.read_response(331).and_then(|_| {
            let pass_command = format!("PASS {}\r\n", password);
            try!(self.write_str(&pass_command));
            try!(self.read_response(230));
            Ok(())
        })
    }

    /// Change the current directory to the path specified.
    pub fn change_dir(&mut self, path: &str) -> Result<()> {
        let cwd_command = format!("CWD {}\r\n", path);

        try!(self.write_str(&cwd_command));
        try!(self.read_response(250));
        Ok(())
    }

    /// Move the current directory to the parent directory.
    pub fn change_dir_to_parent(&mut self) -> Result<()> {
        let cdup_command = format!("CDUP\r\n");

        try!(self.write_str(&cdup_command));
        try!(self.read_response(250));
        Ok(())
    }

    /// Gets the current directory
    pub fn current_dir(&mut self) -> Result<String> {
        fn index_of(string: &str, ch: char) -> isize {
            let mut i = -1;
            let mut index = 0;
            for c in string.chars() {
                if c == ch {
                    i = index;
                    return i;
                }
                index += 1;
            }
            return i;
        }

        fn last_index_of(string: &str, ch: char) -> isize {
            let mut i = -1;
            let mut index = 0;
            for c in string.chars() {
                if c == ch {
                    i = index;
                }
                index += 1;
            }
            return i;
        }

        let pwd_command = format!("PWD\r\n");

        try!(self.write_str(&pwd_command));
        self.read_response(257).and_then(|(_, line)| {
            let begin = index_of(&line, '"');
            let end = last_index_of(&line, '"');

            if begin == -1 || end == -1 {
                let cause = format!("Invalid PWD Response: {}", line);
                return Err(Error::new(ErrorKind::Other, cause));
            }
            let b = begin as usize;
            let e = end as usize;

            return Ok(line[b + 1..e].to_string());
        })
    }

    /// This does nothing. This is usually just used to keep the connection open.
    pub fn noop(&mut self) -> Result<()> {
        let noop_command = format!("NOOP\r\n");
        try!(self.write_str(&noop_command));
        try!(self.read_response(200));
        Ok(())
    }

    /// This creates new directories on the server.
    pub fn make_dir(&mut self, pathname: &str) -> Result<()> {
        let mkdir_command = format!("MKD {}\r\n", pathname);
        try!(self.write_str(&mkdir_command));
        try!(self.read_response(257));
        Ok(())
    }

    /// Runs the PASV command.
    fn pasv(&mut self) -> Result<TcpStream> {
        try!(self.write_str("PASV\r\n"));

        // PASV response format : 227 Entering Passive Mode (h1,h2,h3,h4,p1,p2).
        self.read_response(227).and_then(|(_, line)| {
            let vec = line.split(",").collect::<Vec<_>>();
            if vec.len() != 6 {
                return Err(Error::new(ErrorKind::InvalidData, format!("Invalid PASV response: {}", line)));
            }

            match (vec[4].parse::<u8>(), vec[5].parse::<u8>()) {
                (Ok(msb), Ok(lsb)) => {
                    let port = ((msb as u16) << 8) + lsb as u16;
                    let addr = format!("{}.{}.{}.{}:{}", vec[0], vec[1], vec[2], vec[3], port);
                    TcpStream::connect(&*addr)
                },
                _ => Err(Error::new(ErrorKind::InvalidData, format!("Invalid PASV response: {}", line)))
            }
        })
    }

    /// Quits the current FTP session.
    pub fn quit(&mut self) -> Result<()> {
        let quit_command = format!("QUIT\r\n");
        try!(self.write_str(&quit_command));
        try!(self.read_response(221));
        Ok(())
    }

    /// Retrieves the file name specified from the server.
    /// This method is a more complicated way to retrieve a file.
    /// The reader returned should be dropped.
    /// Also you will have to read the response to make sure it has the correct value.
    pub fn retr(&mut self, file_name: &str) -> Result<BufReader<TcpStream>> {
        let retr_command = format!("RETR {}\r\n", file_name);
        let data_stream = BufReader::new(try!(self.pasv()));

        try!(self.write_str(&retr_command));
        self.read_response(150).and_then(|_| Ok(data_stream))
    }

    fn simple_retr_(&mut self, file_name: &str) -> Result<Cursor<Vec<u8>>> {
        let mut data_stream = match self.retr(file_name) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        let buffer: &mut Vec<u8> = &mut Vec::new();
        loop {
            let mut buf = [0; 256];
            let len = try!(data_stream.read(&mut buf));
            if len == 0 {
                break;
            }
            try!(buffer.write(&buf[0..len]));
        }

        drop(data_stream);

        Ok(Cursor::new(buffer.clone()))
    }

    /// Simple way to retr a file from the server. This stores the file in memory.
    pub fn simple_retr(&mut self, file_name: &str) -> Result<Cursor<Vec<u8>>> {
        let r = try!(self.simple_retr_(file_name));
        try!(self.read_response(226));
        Ok(r)
    }

    /// Removes the remote pathname from the server.
    pub fn remove_dir(&mut self, pathname: &str) -> Result<()> {
        let rmd_command = format!("RMD {}\r\n", pathname);
        try!(self.write_str(&rmd_command));
        try!(self.read_response(250));
        Ok(())
    }

    fn stor_<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<()> {
        let stor_command = format!("STOR {}\r\n", filename);
        let mut data_stream = BufWriter::new(try!(self.pasv()));

        try!(self.write_str(&stor_command));
        try!(self.read_response(150));

        try!(copy(r, &mut data_stream));
        Ok(())
    }

    /// This stores a file on the server.
    pub fn stor<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<()> {
        try!(self.stor_(filename, r));
        try!(self.read_response(226));
        Ok(())
    }

    /// Retrieve single line response
    pub fn read_response(&mut self, expected_code: u32) -> Result<(u32, String)> {
        let mut line = String::new();
        try!(self.reader.read_line(&mut line));
        if line.len() < 5 {
            return Err(Error::new(ErrorKind::Other, "error: could not read reply code".to_owned()))
        }

        let code: u32 = try!(line[0..3].parse().or_else(|err| {
            Err(Error::new(ErrorKind::Other, format!("error: could not parse reply code: {}", err)))
        }));

        // multiple line reply
        // loop while the line does not begin with the code and a space
        let expected = format!("{} ", &line[0..3]);
        while line.len() < 5 || line[0..4] != expected {
            line.clear();
            try!(self.reader.read_line(&mut line));
        }

        if code == expected_code {
            Ok((code, line))
        } else {
            Err(Error::new(ErrorKind::Other, format!("Invalid response: {} {}", code, line)))
        }
    }
}
