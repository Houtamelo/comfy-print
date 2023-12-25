use super::utils::*;

use std::io::Write;

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_sync(Message::Standard(input));
}

pub fn _print(input: String) {
	_comfy_sync(Message::Standard(input));
}

pub fn _eprint(input: String) {
	_comfy_sync(Message::Error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_sync(Message::Error(input));
}

#[allow(unused_must_use)]
pub fn _comfy_sync(mut msg: Message) {
	let default_print = std::panic::catch_unwind(
		|| {
			match &msg {
				Message::Standard(msg) => print!("{}", msg),
				Message::Error(msg) => eprint!("{}", msg),
			}
		});

	if let Ok(()) = default_print {
		return;
	}

	match &mut msg {
		Message::Standard(msg) => msg,
		Message::Error(msg) => msg,
	}.insert_str(0, "`std::print!` panicked, comfy_print actually saved you! Well maybe, we'll try to get a blocking lock on std(out/err).\n");

	match msg {
		Message::Standard(msg) => {
			let mut std_out = std::io::stdout().lock();
			std_out.write_all(msg.as_bytes())
				   .inspect_err(|err| print_stderr(err));
			std_out.flush()
				   .inspect_err(|err| print_stderr(err));
		}
		Message::Error(msg) => {
			let mut std_err = std::io::stderr().lock();
			std_err.write_all(msg.as_bytes())
				   .inspect_err(|err| print_stdout(err));
			std_err.flush()
				   .inspect_err(|err| print_stdout(err));
		}
	}
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::sync::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::sync::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::sync::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::sync::_eprint(std::format!($($arg)*));
	}};
}

#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::sync::_eprintln("\n")
	};
	($($arg:tt)*) => {{
		$crate::sync::_eprintln(std::format!($($arg)*));
	}};
}
