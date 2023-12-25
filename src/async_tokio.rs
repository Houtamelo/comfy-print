use super::utils::*;

use std::io::Write;
use std::ops::DerefMut;
use std::thread;

use tokio::io::AsyncWriteExt;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{MutexGuard, Mutex};

static TOKIO_RUNTIME: Mutex<Option<Runtime>> = Mutex::<Option<Runtime>>::const_new(None);

pub fn _print(input: String) {
	_comfy_async_tokio(Message::Standard(input));
}

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_async_tokio(Message::Standard(input));
}

pub fn _eprint(input: String) {
	_comfy_async_tokio(Message::Error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_async_tokio(Message::Error(input));
}

pub fn _comfy_async_tokio(mut msg: Message) {
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

	(match &mut msg {
		Message::Standard(s) => s,
		Message::Error(e) => e,
	}).insert_str(0, "`std::print!` panicked, comfy_print actually saved you! Well maybe, we'll spawn a Tokio thread to queue the print.\n");

	match TOKIO_RUNTIME.try_lock() {
		Ok(guard) => { write_guard(guard, msg); }
		Err(_) => { wait_for_runtime_lock(msg); }
	}
}

#[allow(unused_must_use)]
pub fn wait_for_runtime_lock(msg: Message) {
	thread::Builder::new().name("thread_comfy_print: TOKIO_RUNTIME is blocked, waiting for lock".to_owned()).spawn(
		move || {
			let guard: MutexGuard<Option<Runtime>> = TOKIO_RUNTIME.blocking_lock();
			write_guard(guard, msg);
		}).inspect_err(|err| print_stderr(err));
}

pub fn write_guard(mut guard: MutexGuard<Option<Runtime>>, msg: Message) {
	match guard.deref_mut() {
		Some(runtime) => {
			write_runtime(runtime, msg);
		}
		None => {
			match Builder::new_current_thread().enable_io().build() {
				Ok(mut runtime) => {
					write_runtime(&mut runtime, msg);
					*guard.deref_mut() = Some(runtime);
				},
				Err(err) => {
					write_std_thread(Message::Error(format!(
						"comfy_print:: Error while trying to create Tokio::Runtime.\n\
						Creation was attempted because the mutex was empty.\n\
						Inner error: {err}")));
				}
			};
		}
	}
}

#[allow(unused_must_use)]
pub fn write_runtime(runtime: &mut Runtime, msg: Message) {
	runtime.spawn(
		async move {
			match msg {
				Message::Standard(msg) => {
					let mut std_out = tokio::io::stdout();
					std_out.write_all(msg.as_bytes()).await
						   .inspect_err(|err| print_stderr(err));
					std_out.flush().await
						   .inspect_err(|err| print_stderr(err));
				}
				Message::Error(msg) => {
					let mut std_err = tokio::io::stderr();
					std_err.write_all(msg.as_bytes()).await
						   .inspect_err(|err| print_stdout(err));
					std_err.flush().await
						   .inspect_err(|err| print_stdout(err));
				}
			}
		});
}

#[allow(unused_must_use)]
pub fn write_std_thread(msg: Message) {
	thread::Builder::new().spawn(
		move || {
			match msg {
				Message::Standard(msg) => {
					let mut std_out = std::io::stdout();
					std_out.write_all(msg.as_bytes())
						   .inspect_err(|err| print_stderr(err));
					std_out.flush()
						   .inspect_err(|err| print_stderr(err));
				}
				Message::Error(msg) => {
					let mut std_err = std::io::stderr();
					std_err.write_all(msg.as_bytes())
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
        $crate::async_tokio::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::async_tokio::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::async_tokio::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::async_tokio::_eprint(std::format!($($arg)*));
	}};
}

#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::async_tokio::_eprintln("\n")
	};
	($($arg:tt)*) => {{
		$crate::async_tokio::_eprintln(std::format!($($arg)*));
	}};
}
