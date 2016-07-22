use std::io::{Error, ErrorKind, Read, Result, BufRead, BufReader, BufWriter, Cursor, Write, copy};
use std::net::{TcpStream, SocketAddr};
use std::string::String;
use std::str::FromStr;
use std::net::ToSocketAddrs;
use regex::Regex;
use chrono::{DateTime, UTC};
use chrono::offset::TimeZone;
#[cfg(feature = "secure")]
use openssl::ssl::{Ssl, SslContext, SslMethod, SslStream, IntoSsl};
use super::data_stream::DataStream;
use super::status;
use super::types::FileType;

lazy_static! {
    // This regex extracts IP and Port details from PASV command response.
    // The regex looks for the pattern (h1,h2,h3,h4,p1,p2).
    static ref PORT_RE: Regex = Regex::new(r"\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)").unwrap();

    // This regex extracts modification time from MDTM command response.
    static ref MDTM_RE: Regex = Regex::new(r"\b(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})\b").unwrap();

    // This regex extracts file size from SIZE command response.
    static ref SIZE_RE: Regex = Regex::new(r"\s+(\d+)\s*$").unwrap();
}

#[cfg(feature = "secure")]
lazy_static! {
    // Shared SSL context
    static ref SSL_CONTEXT: SslContext = match SslContext::new(SslMethod::Sslv23) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };
}

/// Stream to interface with the FTP server. This interface is only for the command stream.
#[derive(Debug)]
pub struct FtpStream {
    reader: BufReader<DataStream>,
}

impl FtpStream {
    /// Creates an FTP Stream.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<FtpStream> {
        match TcpStream::connect(addr) {
            Ok(stream) => {
                let mut ftp_stream = FtpStream {
                    reader: BufReader::new(DataStream::Tcp(stream)),
                };

                try!(ftp_stream.read_response(status::READY));
                Ok(ftp_stream)
            },
            Err(e) => Err(e)
        }
    }

    /// Switch to secure mode if possible. If the connection is already
    /// secure does nothing.
    ///
    /// ## Panics
    ///
    /// Panics if the plain TCP connection cannot be switched to TLS mode.
    ///
    /// ## Example
    ///
    /// ```
    /// use ftp::FtpStream;
    /// let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();
    /// // Switch to the secure mode
    /// let (mut ftp_stream, _) = ftp_stream.secure();
    /// // Do all secret things
    /// let _ = ftp_stream.quit();
    /// ```
    ///
    #[cfg(feature = "secure")]
    pub fn secure(mut self) -> (FtpStream, Result<()>) {
        // Initialize SSL with a default context and make secure the stream.
        let ssl = match Ssl::new(&SSL_CONTEXT) {
            Ok(ssl) => ssl,
            Err(e) => panic!("error: cannot create SSL context: {}", e)
        };
        self.secure_with_ssl(ssl)
    }

    /// Switch to a secure mode if possible, using a provided SSL configuration.
    /// This method does nothing if the connect is already secured.
    ///
    /// ## Panics
    ///
    /// Panics if the plain TCP connection cannot be switched to TLS mode.
    ///
    /// ## Example
    ///
    /// ```
    /// use ftp::FtpStream;
    /// use openssl::ssl::*;
    ///
    /// // Create an SslContext with a custom cert.
    /// let mut ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    /// let _ = ctx.set_CA_file("/path/to/a/cert.pem").unwrap();
    /// let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();
    /// let (mut ftp_stream, _) = ftp_stream.secure_with_ssl(ctx);
    /// ```
    #[cfg(feature = "secure")]
    pub fn secure_with_ssl<S: IntoSsl>(mut self, ssl: S) -> (FtpStream, Result<()>) {
        let secured = self.reader.get_ref().is_ssl();
        if secured {
            (self, Ok(()))
        }
        else {
            // Ask the server to start securing data
            let auth_command = String::from("AUTH TLS\r\n");
            if let Err(e) = self.write_str(&auth_command) {
                return (self, Err(e));
            }

            if let Err(e) = self.read_response(status::AUTH_OK) {
                return (self, Err(e));
            }

            let stream = match SslStream::connect(ssl, self.reader.into_inner().into_tcp_stream()) {
                Ok(stream) => stream,
                Err(e) => panic!("error: cannot open SSL connection: {}", e)
            };

            let mut secured_ftp_tream = FtpStream {
                reader: BufReader::new(DataStream::Ssl(stream)),
            };

            // Set protection buffer size
            let pbsz_command = format!("PBSZ 0\r\n");
            if let Err(e) = secured_ftp_tream.write_str(&pbsz_command) {
                return (secured_ftp_tream, Err(e));
            }

            if let Err(e) = secured_ftp_tream.read_response(status::COMMAND_OK) {
                return (secured_ftp_tream, Err(e));
            }

            // Change the level of data protectio to Private
            let prot_command = String::from("PROT P\r\n");
            if let Err(e) = secured_ftp_tream.write_str(&prot_command) {
                return (secured_ftp_tream, Err(e));
            }

            if let Err(e) = secured_ftp_tream.read_response(status::COMMAND_OK) {
                return (secured_ftp_tream, Err(e));
            }

            (secured_ftp_tream, Ok(()))
        }
    }

    /// Switch to insecure mode. If the connection is already
    /// insecure does nothing.
    ///
    /// ## Example
    ///
    /// ```
    /// use ftp::FtpStream;
    /// let mut ftp_stream = FtpStream::connect("127.0.0.1:21").unwrap();
    /// // Switch to the secure mode
    /// let (mut ftp_stream, _) = ftp_stream.secure();
    /// // Do all secret things
    /// // Switch back to the insecure mode
    /// let (mut ftp_stream, _) = ftp_stream.insecure();
    /// // Do all public things
    /// let _ = ftp_stream.quit();
    /// ```
    ///
    #[cfg(feature = "secure")]
    pub fn insecure(mut self) -> (FtpStream, Result<()>) {
        let secured = self.reader.get_ref().is_ssl();
        if secured {
            // Ask the server to stop securing data
            let ccc_command = String::from("CCC\r\n");
            if let Err(e) = self.write_str(&ccc_command) {
                return (self, Err(e));
            }

            if let Err(e) = self.read_response(status::COMMAND_OK) {
                return (self, Err(e));
            }

            let plain_ftp_stream = FtpStream {
                reader: BufReader::new(DataStream::Tcp(self.reader.into_inner().into_tcp_stream())),
            };

            (plain_ftp_stream, Ok(()))
        }
        else {
            (self, Ok(()))
        }
    }

    /// Log in to the FTP server.
    pub fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let user_command = format!("USER {}\r\n", user);
        try!(self.write_str(&user_command));

        self.read_response_in(&[status::LOGGED_IN, status::NEED_PASSWORD]).and_then(|(code, _)| {
            if code == status::NEED_PASSWORD {
                let pass_command = format!("PASS {}\r\n", password);
                try!(self.write_str(&pass_command));
                try!(self.read_response(status::LOGGED_IN));
            }
            Ok(())
        })
    }

    /// Change the current directory to the path specified.
    pub fn cwd(&mut self, path: &str) -> Result<()> {
        let cwd_command = format!("CWD {}\r\n", path);

        try!(self.write_str(&cwd_command));
        try!(self.read_response(status::REQUESTED_FILE_ACTION_OK));
        Ok(())
    }

    /// Move the current directory to the parent directory.
    pub fn cdup(&mut self) -> Result<()> {
        let cdup_command = format!("CDUP\r\n");

        try!(self.write_str(&cdup_command));
        try!(self.read_response(status::REQUESTED_FILE_ACTION_OK));
        Ok(())
    }

    /// Gets the current directory
    pub fn pwd(&mut self) -> Result<String> {
        try!(self.write_str("PWD\r\n"));
        self.read_response(status::PATH_CREATED).and_then(|(_, line)| {
            match (line.find('"'), line.rfind('"')) {
                (Some(begin), Some(end)) if begin < end => Ok(line[begin + 1 .. end].to_string()),
                _ => {
                    let cause = format!("Invalid PWD Response: {}", line);
                    Err(Error::new(ErrorKind::Other, cause))
                }
            }
        })
    }

    /// This does nothing. This is usually just used to keep the connection open.
    pub fn noop(&mut self) -> Result<()> {
        let noop_command = format!("NOOP\r\n");
        try!(self.write_str(&noop_command));
        try!(self.read_response(status::COMMAND_OK));
        Ok(())
    }

    /// This creates a new directory on the server.
    pub fn mkdir(&mut self, pathname: &str) -> Result<()> {
        let mkdir_command = format!("MKD {}\r\n", pathname);
        try!(self.write_str(&mkdir_command));
        try!(self.read_response(status::PATH_CREATED));
        Ok(())
    }

    /// Runs the PASV command.
    fn pasv(&mut self) -> Result<SocketAddr> {
        try!(self.write_str("PASV\r\n"));

        // PASV response format : 227 Entering Passive Mode (h1,h2,h3,h4,p1,p2).
        self.read_response(status::PASSIVE_MODE).and_then(|(_, line)| {
            match PORT_RE.captures(&line) {
                Some(caps) => {
                    // If the regex matches we can be sure groups contains numbers
                    let (oct1, oct2, oct3, oct4) = (caps[1].parse::<u8>().unwrap(), caps[2].parse::<u8>().unwrap(), caps[3].parse::<u8>().unwrap(), caps[4].parse::<u8>().unwrap());
                    let (msb, lsb) = (caps[5].parse::<u8>().unwrap(), caps[6].parse::<u8>().unwrap());
                    let port = ((msb as u16) << 8) + lsb as u16;
                    let addr = format!("{}.{}.{}.{}:{}", oct1, oct2, oct3, oct4, port);

                    match SocketAddr::from_str(&addr) {
                        Ok(addr) => Ok(addr),
                        Err(e) => Err(Error::new(ErrorKind::Other, e))
                    }
                },
                None => {
                    Err(Error::new(ErrorKind::InvalidData, format!("Invalid PASV response: {}", line)))
                }
            }
        })
    }

    // Execute command which send data back in a separate stream
    #[cfg(not(feature = "secure"))]
    fn data_command(&mut self, cmd: &str) -> Result<DataStream> {
        let addr = self.pasv();
        match addr {
            Ok(addr) => {
                // First send command, then open data connection otherwise
                // the peer might not do a full accept (with SSL handshake if PROT P
                try!(self.write_str(cmd));

                match TcpStream::connect(addr) {
                    Ok(stream) => Ok(DataStream::Tcp(stream)),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    #[cfg(feature = "secure")]
    fn data_command(&mut self, cmd: &str) -> Result<DataStream> {
        let addr = self.pasv();
        match addr {
            Ok(addr) => {
                // First send command, then open data connection otherwise
                // the peer might not do a full accept (with SSL handshake if PROT P
                try!(self.write_str(cmd));

                match TcpStream::connect(addr) {
                    Ok(stream) => {
                        if self.reader.get_ref().is_ssl() {
                            // Secure the connection
                            let ssl = match Ssl::new(&SSL_CONTEXT) {
                                Ok(ssl) => ssl,
                                Err(e) => return Err(Error::new(ErrorKind::Other, e))
                            };

                            match SslStream::connect(ssl, stream) {
                                Ok(stream) => {
                                    Ok(DataStream::Ssl(stream))
                                },
                                Err(e) => Err(Error::new(ErrorKind::Other, e))
                            }
                        }
                        else {
                            Ok(DataStream::Tcp(stream))
                        }
                    },
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    /// Sets the type of file to be transferred. That is the implementation
    /// of `TYPE` command.
    pub fn transfer_type(&mut self, file_type: FileType) -> Result<()> {
        let type_command = format!("TYPE {}\r\n", file_type.to_string());
        try!(self.write_str(&type_command));
        try!(self.read_response(status::COMMAND_OK));
        Ok(())
    }

    /// Quits the current FTP session.
    pub fn quit(&mut self) -> Result<()> {
        let quit_command = format!("QUIT\r\n");
        try!(self.write_str(&quit_command));
        try!(self.read_response(status::CLOSING));
        Ok(())
    }

    /// Retrieves the file name specified from the server.
    /// This method is a more complicated way to retrieve a file.
    /// The reader returned should be dropped.
    /// Also you will have to read the response to make sure it has the correct value.
    pub fn get(&mut self, file_name: &str) -> Result<BufReader<DataStream>> {
        let retr_command = format!("RETR {}\r\n", file_name);
        let data_stream = BufReader::new(try!(self.data_command(&retr_command)));
        self.read_response(status::ABOUT_TO_SEND).and_then(|_| Ok(data_stream))
    }

    /// Renames the file from_name to to_name
    pub fn rename(&mut self, from_name: &str, to_name: &str) -> Result<()> {
        let rnfr_command = format!("RNFR {}\r\n", from_name);
        try!(self.write_str(&rnfr_command));

        self.read_response(status::REQUEST_FILE_PENDING).and_then(|_| {
            let rnto_command = format!("RNTO {}\r\n", to_name);
            try!(self.write_str(&rnto_command));
            try!(self.read_response(status::REQUESTED_FILE_ACTION_OK));
            Ok(())
        })
    }

    /// The implementation of `RETR` command where `filename` is the name of the file
    /// to download from FTP and `reader` is the function which operates with the
    /// data stream opened.
    ///
    /// ```ignore
    /// let result = conn.retr("take_this.txt", |stream| {
    ///   let mut file = File::create("store_here.txt").unwrap();
    ///   let mut buf = [0; 2048];
    ///
    ///   loop {
    ///     match stream.read(&mut buf) {
    ///       Ok(0) => break,
    ///       Ok(n) => file.write_all(&buf[0..n]).unwrap(),
    ///       Err(err) => return Err(err)
    ///     };
    ///   }
    ///
    ///   Ok(())
    /// });
    /// ```
    pub fn retr<F>(&mut self, filename: &str, reader: F) -> Result<()>
    where F: Fn(&mut Read) -> Result<()> {
        let retr_command = format!("RETR {}\r\n", filename);
        let mut data_stream = BufReader::new(try!(self.data_command(&retr_command)));
        self.read_response_in(&[status::ABOUT_TO_SEND, status::ALREADY_OPEN]).and_then(|_| {
            let result = reader(&mut data_stream);
            drop(data_stream);
            try!(self.read_response(status::CLOSING_DATA_CONNECTION));

            result
        })
    }

    fn simple_retr_(&mut self, file_name: &str) -> Result<Cursor<Vec<u8>>> {
        let mut data_stream = match self.get(file_name) {
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
        try!(self.read_response(status::CLOSING_DATA_CONNECTION));
        Ok(r)
    }

    /// Removes the remote pathname from the server.
    pub fn rmdir(&mut self, pathname: &str) -> Result<()> {
        let rmd_command = format!("RMD {}\r\n", pathname);
        try!(self.write_str(&rmd_command));
        try!(self.read_response(status::REQUESTED_FILE_ACTION_OK));
        Ok(())
    }

    /// Remove the remote file from the server.
    pub fn rm(&mut self, filename: &str) -> Result<()> {
        let rm_command = format!("DELE {}\r\n", filename);
        try!(self.write_str(&rm_command));
        try!(self.read_response(status::REQUESTED_FILE_ACTION_OK));
        Ok(())
    }

    fn put_file<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<()> {
        let stor_command = format!("STOR {}\r\n", filename);
        let mut data_stream = BufWriter::new(try!(self.data_command(&stor_command)));
        try!(self.read_response_in(&[status::ALREADY_OPEN, status::ABOUT_TO_SEND]));

        try!(copy(r, &mut data_stream));
        Ok(())
    }

    /// This stores a file on the server.
    pub fn put<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<()> {
        try!(self.put_file(filename, r));
        try!(self.read_response(status::CLOSING_DATA_CONNECTION));
        Ok(())
    }

    /// Execute a command which returns list of strings in a separate stream
    fn list_command(&mut self, cmd: String, open_code: u32, close_code: u32) -> Result<Vec<String>> {
        let mut data_stream = BufReader::new(try!(self.data_command(&cmd)));
        try!(self.read_response_in(&[open_code, status::ALREADY_OPEN]));

        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        loop {
            match data_stream.read_to_string(&mut line) {
                Ok(0) => break,
                Ok(_) => lines.extend(line.split("\r\n").into_iter().map(|s| String::from(s)).filter(|s| s.len() > 0)),
                Err(err) => return Err(err),
            };
        }

        try!(self.read_response(close_code));

        Ok(lines)
    }

    /// Execute `LIST` command which returns the detailed file listing in human readable format.
    /// If `pathname` is omited then the list of files in the current directory will be
    /// returned otherwise it will the list of files on `pathname`.
    pub fn list(&mut self, pathname: Option<&str>) -> Result<Vec<String>> {
        let command = match pathname {
            Some(path) => format!("LIST {}\r\n", path),
            None => String::from("LIST\r\n"),
        };

        self.list_command(command, status::ABOUT_TO_SEND, status::CLOSING_DATA_CONNECTION)
    }

    /// Execute `NLST` command which returns the list of file names only.
    /// If `pathname` is omited then the list of files in the current directory will be
    /// returned otherwise it will the list of files on `pathname`.
    pub fn nlst(&mut self, pathname: Option<&str>) -> Result<Vec<String>> {
        let command = match pathname {
            Some(path) => format!("NLST {}\r\n", path),
            None => String::from("NLST\r\n"),
        };

        self.list_command(command, status::ABOUT_TO_SEND, status::CLOSING_DATA_CONNECTION)
    }

    /// Retrieves the modification time of the file at `pathname` if it exists.
    /// In case the file does not exist `None` is returned.
    pub fn mdtm(&mut self, pathname: &str) -> Result<Option<DateTime<UTC>>> {
        let mdtm_command = format!("MDTM {}\r\n", pathname);
        try!(self.write_str(&mdtm_command));
        let (_, line) = try!(self.read_response(status::FILE));

        match MDTM_RE.captures(&line) {
            Some(caps) => {
                let (year, month, day) = (caps[1].parse::<i32>().unwrap(), caps[2].parse::<u32>().unwrap(), caps[3].parse::<u32>().unwrap());
                let (hour, minute, second) = (caps[4].parse::<u32>().unwrap(), caps[5].parse::<u32>().unwrap(), caps[6].parse::<u32>().unwrap());
                Ok(Some(UTC.ymd(year, month, day).and_hms(hour, minute, second)))
            },
            None => Ok(None)
        }
    }

    /// Retrieves the size of the file in bytes at `pathname` if it exists.
    /// In case the file does not exist `None` is returned.
    pub fn size(&mut self, pathname: &str) -> Result<Option<usize>> {
        let size_command = format!("SIZE {}\r\n", pathname);
        try!(self.write_str(&size_command));
        let (_, line) = try!(self.read_response(status::FILE));

        match SIZE_RE.captures(&line) {
            Some(caps) => Ok(Some(caps[1].parse().unwrap())),
            None => Ok(None)
        }
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        let stream = self.reader.get_mut();

        if cfg!(feature = "debug_print") {
            print!("CMD {}", s);
        }

        return stream.write_fmt(format_args!("{}", s));
    }

    pub fn read_response(&mut self, expected_code: u32) -> Result<(u32, String)> {
        self.read_response_in(&[expected_code])
    }

    /// Retrieve single line response
    pub fn read_response_in(&mut self, expected_code: &[u32]) -> Result<(u32, String)> {
        let mut line = String::new();
        try!(self.reader.read_line(&mut line));

        if cfg!(feature = "debug_print") {
            print!("FTP {}", line);
        }

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

            if cfg!(feature = "debug_print") {
                print!("FTP {}", line);
            }
        }

        if expected_code.into_iter().any(|ec| code == *ec) {
            Ok((code, line))
        } else {
            Err(Error::new(ErrorKind::Other, format!("Expected code {:?}, got response: {}", expected_code, line)))
        }
    }
}
