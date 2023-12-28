use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub enum Message {
	Standard(String),
	Error(String),
}

impl Deref for Message {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		return match self {
			Message::Standard(s) => s.as_str(),
			Message::Error(e) => e.as_str(),
		};
	}
}

impl Message {
	pub fn output_kind(&self) -> OutputKind {
		return match self {
			Message::Standard(_) => OutputKind::Stdout,
			Message::Error(_) => OutputKind::Stderr,
		};
	}
}


#[derive(Debug, Copy, Clone)]
pub enum OutputKind {
	Stdout,
	Stderr,
}

impl Display for Message {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Message::Standard(s) => write!(f, "{}", s),
			Message::Error(e) => write!(f, "{}", e),
		}
	}
}