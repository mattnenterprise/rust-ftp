# Changelog

This project follows semantic versioning.

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
- ...

### [Unreleased from branch list_commands]
- [changed] Separate main lib file and FTP stream implementation.
- [changed] Regex is used to parse PASV response.
- [added] The implementation of LIST command. See method `list`.
- [added] The implementation of NLST command. See method `nlst`.
- [added] The implementation of MDTM command. See method `mdtm`.


### [v0.0.7] (2016-01-11)

- No changelog up to this point

[Unreleased]: https://github.com/coredump-ch/coredumpbot/compare/761deb8...HEAD
[0.0.7]: https://github.com/mattnenterprise/rust-ftp/compare/ef996f0...761deb8
