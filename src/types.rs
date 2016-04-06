//! The set of valid values for FTP commands


/// Text Format Control used in `TYPE` command
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatControl {
	/// Default text format control (is NonPrint)
	Default,
	/// Non-print (not destined for printing)
	NonPrint,
	/// Telnet format control (\<CR\>, \<FF\>, etc.)
	Telnet,
	/// ASA (Fortran) Carriage Control
	Asa,
}


/// File Type used in `TYPE` command
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
	/// ASCII text (the argument is the text format control)
	Ascii(FormatControl),
	/// EBCDIC text (the argument is the text format control)
	Ebcdic(FormatControl),
	/// Image,
	Image,
	/// Binary (the synonym to Image)
	Binary,
	/// Local format (the argument is the number of bits in one byte on local machine)
	Local(u8),
}


impl ToString for FormatControl {
	fn to_string(&self) -> String {
		match self {
			&FormatControl::Default | &FormatControl::NonPrint => String::from("N"),
			&FormatControl::Telnet => String::from("T"),
			&FormatControl::Asa => String::from("C"),
		}
	}
}


impl ToString for FileType {
	fn to_string(&self) -> String {
		match self {
			&FileType::Ascii(ref fc) => format!("A {}", fc.to_string()),
			&FileType::Ebcdic(ref fc) => format!("E {}", fc.to_string()),
			&FileType::Image | &FileType::Binary => String::from("I"),
			&FileType::Local(ref bits) => format!("L {}", bits),
		}
	}
}
