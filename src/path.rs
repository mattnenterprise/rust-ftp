
Enum FtpPathType {
    directory,
    file,
    linc,
}

Struct FtpPath {
    home: String,
    absolute_path: String,
    path_type: FtpPathType,
}
