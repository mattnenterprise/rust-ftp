# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.


### [Unreleased]

- [changed] The `FTPStream` struct was renamed to `FtpStream` (#17)
- [added] The `host` parameter for `FtpStream` now accepts any type that
  implements `Into<String>` (#13)
- [changed] FTP return code type changed from `isize` to `u32` (#18)
- [changed] Type of port number returned by `pasv` changed from `isize`
  to `u32` (#18)
- [changed] Improved error handling (#21)
- [added] Ability to rename files on the server
- ...

### [Unreleased from branch list_commands]
- [changed] Separate main lib file and FTP stream implementation.
- [changed] Regex is used to parse PASV response.
- [added] The implementation of LIST command. See method `FtpStream::list`.
- [added] The implementation of NLST command. See method `FtpStream::nlst`.
- [added] The implementation of MDTM command. See method `FtpStream::mdtm`.
- [added] The implementation of SIZE command. See method `FtpStream::size`.

### [Unreleased from branch retr_and_type]
- [added] The implementation of RETR command. See method `FtpStream::retr`.
- [added] The implementation of TYPE command. See method `FtpStream::transfer_type`.

### [Unreleased from branch ftps_support]
- [added] Feature `secure` to enable FTPS support. Disabled be default.
- [added] Feature `debug_print` to print command and responses to STDOUT. Disabled be default.
- [added] DataStream which hides the underlying secure or insecure TCP stream.
- [changed] Methods return `DataStream` instead of `TcpStream`.
- [changed] Method `pasv` returns only IP and port and do not open new TCP stream.
- [added] Method `data_command` which issues `pasv` to open the new `DataStream`.
- [added] Methods `secure` and `insecure` to switch between secure and insecure modes.


### [v0.0.7] (2016-01-11)

- No changelog up to this point

[Unreleased]: https://github.com/coredump-ch/coredumpbot/compare/761deb8...HEAD
[0.0.7]: https://github.com/mattnenterprise/rust-ftp/compare/ef996f0...761deb8
