#[cfg(feature = "secure")]

extern crate ftp;
extern crate openssl;

use ftp::FtpStream;
use openssl::ssl::{
    SslContext,
    SslMethod,
    SSL_OP_NO_SSLV2,
    SSL_OP_NO_SSLV3,
    SSL_OP_NO_COMPRESSION,
};

fn main() {
    let mut builder = SslContext::builder(SslMethod::tls()).unwrap();
    builder.set_certificate_file("./tests/test.crt", openssl::x509::X509_FILETYPE_PEM).unwrap();
    builder.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    builder.set_cipher_list("ALL!EXPORT!EXPORT40!EXPORT56!aNULL!LOW!RC4@STRENGTH").unwrap();
    let ctx = builder.build();
    let result = FtpStream::connect("127.0.0.1:21")
        .and_then(|mut client| client.login("anonymous", "").map(|_| client))
        .and_then(|client| client.into_secure(ctx))
        .and_then(|mut client| client.list(None));
    match result {
        Ok(dir)  => {
            for file in dir.iter() {
                println!("{}", file);
            }
        },
        Err(err) => println!("Error: {:?}", err)
    }
}
