use super::utils::*;

use std::io::Write;
use std::thread;

pub fn _print(input: String) {
	_comfy_async_std(Message::Standard(input));
}

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_async_std(Message::Standard(input));
}

pub fn _eprint(input: String) {
	_comfy_async_std(Message::Error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_async_std(Message::Error(input));
}

pub fn _comfy_async_std(mut msg: Message) {
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
		Message::Standard(s) => s,
		Message::Error(e) => e,
	}.insert_str(0, "`std::print!` panicked, comfy_print actually saved you! Well maybe, we'll spawn a std thread to queue the print.\n");

	write_thread(msg);
}

#[allow(unused_must_use)]
pub fn write_thread(msg: Message) {
	thread::Builder::new().spawn(
		move || {
			match msg {
				Message::Standard(msg) => {
					let mut std_out = std::io::stdout();
					std_out.write_fmt(format_args!("{}", msg))
						   .inspect_err(|err| print_stderr(err));
					std_out.flush()
						   .inspect_err(|err| print_stderr(err));
				}
				Message::Error(msg) => {
					let mut std_err = std::io::stderr();
					std_err.write_fmt(format_args!("{}", msg))
						   .inspect_err(|err| print_stdout(err));
					std_err.flush()
						   .inspect_err(|err| print_stdout(err));
				}
			}
		}).inspect_err(|err| print_stderr(err));
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::async_std::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::async_std::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::async_std::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::async_std::_eprint(std::format!($($arg)*));
	}};
}

#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::async_std::_eprintln("\n")
	};
	($($arg:tt)*) => {{
		$crate::async_std::_eprintln(std::format!($($arg)*));
	}};
}
