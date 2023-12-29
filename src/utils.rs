use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Debug, Copy, Clone)]
pub enum OutputKind {
	Stdout,
	Stderr,
}

pub struct Message {
	string: String,
	output: OutputKind,
}

impl Message {
	pub fn str(&self) -> &str {
		return self.string.as_str();
	}
	
	pub fn output_kind(&self) -> OutputKind {
		return self.output;
	}

	pub fn standard(string: String) -> Self {
		return Self {
			string,
			output: OutputKind::Stdout,
		};
	}
	
	pub fn error(string: String) -> Self {
		return Self {
			string,
			output: OutputKind::Stderr,
		};
	}
}

impl Display for Message {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		return write!(f, "{}", self.string);
	}
}

pub fn try_write(msg_str: &impl Display, output_kind: OutputKind) -> std::io::Result<()> {
	match output_kind {
		OutputKind::Stdout => {
			let mut stdout = std::io::stdout().lock();
			write!(stdout, "{}", msg_str)?;
			stdout.flush()?;
			Ok(())
		}
		OutputKind::Stderr => {
			let mut stderr = std::io::stderr().lock();
			write!(stderr, "{}", msg_str)?;
			stderr.flush()?;
			Ok(())
		}
	}
}