//! # Message
//! [comfy_print](crate)'s data type for storing messages that failed to be printed.

use std::fmt::{Display, Formatter};
use std::ops::Deref;

/// Which stream to write to.
/// - [Stdout](OutputKind::Stdout) write to [std::io::stdout()](std::io::stdout())
/// - [Stderr](OutputKind::Stderr) write to [std::io::stderr()](std::io::stderr())
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputKind {
	/// Write to [std::io::stdout()](std::io::stdout())
	Stdout,
	/// Write to [std::io::stderr()](std::io::stderr())
	Stderr,
}

/// Structure for storing messages that failed to be printed.
pub struct Message {
	string: String,
	output: OutputKind,
	should_append_line: bool,
}

impl Message {
	pub fn str(&self) -> &str {
		return self.string.deref();
	}
	
	pub fn output_kind(&self) -> OutputKind {
		return self.output;
	}

	pub fn standard(print_me: impl Into<String>) -> Self {
		return Self {
			string: print_me.into(),
			output: OutputKind::Stdout,
			should_append_line: false,
		};
	}
	
	pub fn standard_ln(print_me: impl Into<String>) -> Self {
		return Self {
			string: print_me.into(),
			output: OutputKind::Stdout,
			should_append_line: true,
		};
	}
	
	pub fn error(print_me: impl Into<String>) -> Self {
		return Self {
			string: print_me.into(),
			output: OutputKind::Stderr,
			should_append_line: false,
		};
	}

	pub fn error_ln(print_me: impl Into<String>) -> Self {
		return Self {
			string: print_me.into(),
			output: OutputKind::Stderr,
			should_append_line: true,
		};
	}
}

impl Display for Message {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.should_append_line {
			return write!(f, "{}\n", self.string.deref());
		} else {
			return write!(f, "{}", self.string.deref());
		}
	}
}