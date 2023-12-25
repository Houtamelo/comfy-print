use std::fmt::{Display, Formatter};
use std::io::Write;

pub enum Message {
	Standard(String),
	Error(String),
}

impl Display for Message {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Message::Standard(s) => write!(f, "{}", s),
			Message::Error(e) => write!(f, "{}", e),
		}
	}
}

pub(crate) fn print_stderr<T>(err: T) where T: std::error::Error {
	let mut std_err = std::io::stderr();
	let _ = std_err.write_all(format!("comfy_print internal error: {err}").as_bytes());
	let _ = std_err.flush();
}

/// In case we fail to print to stderr, we'll try to print to stdout
pub(crate) fn print_stdout<T>(err: T) where T: std::error::Error {
	let mut std_out = std::io::stdout();
	let _ = std_out.write_all(format!("comfy_print internal error: {err}").as_bytes());
	let _ = std_out.flush();
}