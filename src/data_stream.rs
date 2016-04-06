use std::io::{Read, Write, Result};
use std::net::TcpStream;
#[cfg(feature = "secure")]
use openssl::ssl::SslStream;


/// Data Stream used for communications
#[derive(Debug)]
pub enum DataStream {
    Tcp(TcpStream),
    #[cfg(feature = "secure")]
	Ssl(SslStream<TcpStream>),
}


impl Read for DataStream {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		match self {
			&mut DataStream::Tcp(ref mut stream) => stream.read(buf),
			#[cfg(feature = "secure")]
			&mut DataStream::Ssl(ref mut stream) => stream.read(buf),
		}
	}
}


impl Write for DataStream {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		match self {
			&mut DataStream::Tcp(ref mut stream) => stream.write(buf),
			#[cfg(feature = "secure")]
			&mut DataStream::Ssl(ref mut stream) => stream.write(buf),
		}
	}

    fn flush(&mut self) -> Result<()> {
		match self {
			&mut DataStream::Tcp(ref mut stream) => stream.flush(),
			#[cfg(feature = "secure")]
			&mut DataStream::Ssl(ref mut stream) => stream.flush(),
		}
    }
}
