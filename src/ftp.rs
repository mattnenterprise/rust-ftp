#![crate_name = "ftp"]
#![crate_type = "lib"]

extern crate regex;

use std::io::{Error, ErrorKind, Read, BufReader, BufWriter , Cursor, Write, copy};
use std::net::TcpStream;
use std::result::Result;
use std::string::String;
use std::str::FromStr;
use regex::Regex;

/// Stream to interface with the FTP server. This interface is only for the command stream.
pub struct FTPStream {
	command_stream: TcpStream,
	pub host: String,
	pub command_port: u16
}

impl FTPStream {

	/// Creates an FTP Stream.
	pub fn connect<S: Into<String>>(host: S, port: u16) -> Result<FTPStream, Error> {
        let host_string = host.into();
		let connect_string = format!("{}:{}", host_string, port);
		let tcp_stream = try!(TcpStream::connect(&*connect_string));
		let mut ftp_stream = FTPStream {
			command_stream: tcp_stream,
			host: host_string,
			command_port: port
		};
		match ftp_stream.read_response(220) {
			Ok(_) => (),
			Err(e) => return Err(Error::new(ErrorKind::Other, e))
		}
		Ok(ftp_stream)
	}

	fn write_str(&mut self, s: &str) -> Result<(), Error> {
		return self.command_stream.write_fmt(format_args!("{}", s));
	}

	/// Log in to the FTP server.
	pub fn login(&mut self, user: &str, password: &str) -> Result<(), String> {
		let user_command = format!("USER {}\r\n", user);
		let pass_command = format!("PASS {}\r\n", password);

		match self.write_str(&user_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(331) {
			Ok(_) => {

				match self.write_str(&pass_command) {
					Ok(_) => (),
					Err(_) => return Err(format!("Write Error"))
				}

				match self.read_response(230) {
					Ok(_) => Ok(()),
					Err(s) => Err(s)
				}
			},
			Err(s) => Err(s)
		}
	}

	/// Change the current directory to the path specified.
	pub fn change_dir(&mut self, path: &str) -> Result<(), String> {
		let cwd_command = format!("CWD {}\r\n", path);

		match self.write_str(&cwd_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(250) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	/// Move the current directory to the parent directory.
	pub fn change_dir_to_parent(&mut self) -> Result<(), String> {
		let cdup_command = format!("CDUP\r\n");

		match self.write_str(&cdup_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(250) {
			Ok(_) => Ok(()),
			Err(s) => Err(s)
		}
	}

	/// Gets the current directory
	pub fn current_dir(&mut self) -> Result<String, String> {
		fn index_of(string: &str, ch: char) -> isize {
			let mut i = -1;
			let mut index = 0;
			for c in string.chars() {
				if c == ch {
					i = index;
					return i
				}
				index+=1;
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
				index+=1;
			}
			return i;
		}
		let pwd_command = format!("PWD\r\n");

		match self.write_str(&pwd_command) {
			Ok(_) => (),
			Err(e) => return Err(format!("{}", e))
		}

		match self.read_response(257) {
			Ok((_, line)) => {
				let begin = index_of(&line, '"');
				let end = last_index_of(&line, '"');

				if begin == -1 || end == -1 {
					return Err(format!("Invalid PWD Response: {}", line))
				}
				let b = begin as usize;
				let e = end as usize;

				return Ok(line[b+1..e].to_string())
			},
			Err(e) => Err(e)
		}
	}

	/// This does nothing. This is usually just used to keep the connection open.
	pub fn noop(&mut self) -> Result<(), String> {
		let noop_command = format!("NOOP\r\n");

		match self.write_str(&noop_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(200) {
			Ok(_) => Ok(()),
			Err(s) => Err(s)
		}
	}

	/// This creates new directories on the server.
	pub fn make_dir(&mut self, pathname: &str) -> Result<(), String> {
		let mkdir_command = format!("MKD {}\r\n", pathname);

		match self.write_str(&mkdir_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(257) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	/// Runs the PASV command.
	pub fn pasv(&mut self) -> Result<(isize), String> {
		let pasv_command = format!("PASV\r\n");

		match self.write_str(&pasv_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		//PASV response format : 227 Entering Passive Mode (h1,h2,h3,h4,p1,p2).

		let response_regex = match Regex::new(r"(.*)\(\d+,\d+,\d+,\d+,(\d+),(\d+)\)(.*)") {
			Ok(re) => re,
    		Err(_) => panic!("Invaid Regex!!"),
		};

		match self.read_response(227) {
			Ok((_, line)) => {
				let caps = response_regex.captures(&line).unwrap();
				let caps_2 = match caps.at(2) {
					Some(s) => s,
					None => return Err(format!("Problems parsing reponse"))
				};
				let caps_3 = match caps.at(3) {
					Some(s) => s,
					None => return Err(format!("Problems parsing reponse"))
				};
				let first_part_port: isize = FromStr::from_str(caps_2).unwrap();
				let second_part_port: isize = FromStr::from_str(caps_3).unwrap();
				Ok((first_part_port*256)+second_part_port)
			},
			Err(s) => Err(s)
		}
	}

	/// Quits the current FTP session.
	pub fn quit(&mut self) -> Result<(isize, String), String> {
		let quit_command = format!("QUIT\r\n");

		match self.write_str(&quit_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(221) {
			Ok((code, message)) => Ok((code, message)),
			Err(message) => Err(message),
		}
	}

	/// Retrieves the file name specified from the server. This method is a more complicated way to retrieve a file. The reader returned should be dropped.
	/// Also you will have to read the response to make sure it has the correct value.
	pub fn retr(&mut self, file_name: &str) -> Result<BufReader<TcpStream>, String> {
		let retr_command = format!("RETR {}\r\n", file_name);

		let port = match self.pasv() {
			Ok(p) => p,
			Err(e) => return Err(e)
		};

		let connect_string = format!("{}:{}", self.host, port);
		let data_stream = BufReader::new(TcpStream::connect(&*connect_string).unwrap());

		match self.write_str(&retr_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(150) {
			Ok(_) => Ok(data_stream),
			Err(e) => Err(e)
		}
	}

	fn simple_retr_(&mut self, file_name: &str) -> Result<Cursor<Vec<u8>>, String> {
		let mut data_stream = match self.retr(file_name) {
			Ok(s) => s,
			Err(e) => return Err(e)
		};

		let buffer: &mut Vec<u8> = &mut Vec::new();
		loop {
			let mut buf = [0; 256];
			let len = match data_stream.read(&mut buf) {
            	Ok(len) => len,
            	//Err(ref e) if e.kind == EndOfFile => break,
            	Err(e) => return Err(format!("{}", e)),
        	};
        	if len == 0 {
				break;
			}
        	match buffer.write(&buf[0..len]) {
        		Ok(_) => (),
        		Err(e) => return Err(format!("{}", e))
        	};
		}

		drop(data_stream);

		Ok(Cursor::new(buffer.clone()))
	}

	/// Simple way to retr a file from the server. This stores the file in memory.
	pub fn simple_retr(&mut self, file_name: &str) -> Result<Cursor<Vec<u8>>, String> {
		let r = match self.simple_retr_(file_name) {
			Ok(reader) => reader,
			Err(e) => return Err(e)
		};

		match self.read_response(226) {
			Ok(_) => Ok(r),
			Err(e) => Err(e)
		}
	}

	/// Removes the remote pathname from the server.
	pub fn remove_dir(&mut self, pathname: &str) -> Result<(), String> {
		let rmd_command = format!("RMD {}\r\n", pathname);

		match self.write_str(&rmd_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(250) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	fn stor_<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<(), String> {
		let stor_command = format!("STOR {}\r\n", filename);

		let port = match self.pasv() {
			Ok(p) => p,
			Err(e) => return Err(e)
		};

		let connect_string = format!("{}:{}", self.host, port);
		let data_stream: &mut BufWriter<TcpStream> = &mut BufWriter::new(TcpStream::connect(&*connect_string).unwrap());

		match self.write_str(&stor_command) {
			Ok(_) => (),
			Err(_) => return Err(format!("Write Error"))
		}

		match self.read_response(150) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match copy(r, data_stream) {
			Ok(_) => {
				drop(data_stream);
				Ok(())
			},
			Err(_) => {
				drop(data_stream);
				Err(format!("Error Writing"))
			}
		}
	}

	/// This stores a file on the server.
	pub fn stor<R: Read>(&mut self, filename: &str, r: &mut R) -> Result<(), String> {
		match self.stor_(filename, r) {
			Ok(_) => (),
			Err(e) => return Err(e)
		};

		match self.read_response(226) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	//Retrieve single line response
	pub fn read_response(&mut self, expected_code: isize) -> Result<(isize, String), String> {
		//Carriage return
		let cr = 0x0d;
		//Line Feed
		let lf = 0x0a;
		let mut line_buffer: Vec<u8> = Vec::new();

		while line_buffer.len() < 2 || (line_buffer[line_buffer.len()-1] != lf && line_buffer[line_buffer.len()-2] != cr) {
				let byte_buffer: &mut [u8] = &mut [0];
				match self.command_stream.read(byte_buffer) {
					Ok(_) => {},
					Err(_) => return Err(format!("Error reading response")),
				}
				line_buffer.push(byte_buffer[0]);
		}

		let response = String::from_utf8(line_buffer).unwrap();
		let chars_to_trim: &[char] = &['\r', '\n'];
		let trimmed_response = response.trim_matches(chars_to_trim);
    	let trimmed_response_vec: Vec<char> = trimmed_response.chars().collect();
    	if trimmed_response_vec.len() < 5 || trimmed_response_vec[3] != ' ' {
    		return Err(format!("Invalid response"));
    	}

    	let v: Vec<&str> = trimmed_response.splitn(2, ' ').collect();
    	let code: isize = FromStr::from_str(v[0]).unwrap();
    	let message = v[1];
    	if code != expected_code {
    		return Err(format!("Invalid response: {} {}", code, message))
    	}
    	Ok((code, message.to_string()))
	}
}
