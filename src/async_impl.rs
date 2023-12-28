use std::fmt::Display;
use std::io::Write;
use std::thread;
use std::thread::JoinHandle;
use parking_lot::FairMutex;

use super::utils::*;

static QUEUE: FairMutex<Vec<Message>> = FairMutex::new(Vec::new());
static ACTIVE_THREAD: FairMutex<Option<JoinHandle<()>>> = FairMutex::new(None);

pub fn _comfy_print_async(msg: Message) {
	let mut queue_guard = QUEUE.lock();
	
	if queue_guard.len() == 0 {
		drop(queue_guard);
		blocking_write_first_in_line(msg);
	} else {
		queue_guard.push(msg);
		drop(queue_guard);
		check_thread();
	}
}

fn check_thread() {
	let Some(mut thread_guard) = ACTIVE_THREAD.try_lock()
			else { return; };

	let is_printing = thread_guard.as_ref().is_some_and(|handle| !handle.is_finished());
	if is_printing { // We already pushed our msg to the queue and there's already a thread printing it, so we can return.
		return;
	}

	match thread::Builder::new().spawn(print_until_empty) {
		Ok(ok) => {
			*thread_guard = Some(ok);
			drop(thread_guard);
		},
		Err(err) => { // We couldn't create a thread, we'll have to block this one
			drop(thread_guard);

			let mut queue_guard = QUEUE.lock();
			queue_guard.push(Message::Error(format!(
				"comfy_print::queue_then_check_thread(): Failed to create a thread to print the queue.\n\
				Error: {err}.")));

			drop(queue_guard);
			print_until_empty();
		}
	}
}

fn print_until_empty() {
	const MAX_RETRIES: u8 = 50;
	
	let mut message_pivot = String::new();
	let mut retries = 0;
	
	loop {
		let mut queue_guard = QUEUE.lock();

		if queue_guard.len() <= 0 {
			drop(queue_guard);
			break;
		}
		
		let msg = queue_guard.remove(0);
		message_pivot.clear();
		message_pivot.push_str(msg.as_ref());
		let output_kind = msg.output_kind();
		drop(queue_guard); // unlock the queue before blocking stdout/err
		
		if let Err(err) = try_write(&message_pivot, output_kind) {
			let mut queue_guard = QUEUE.lock();
			queue_guard.insert(0, Message::Error(
				format!("comfy_print::write_until_empty(): Failed to print first message in queue.\n\
				Error: {err}\n\
				Message: {message_pivot}\n\
				Target output: {output_kind:?}")));
			
			queue_guard.insert(1, msg);
			drop(queue_guard);
			
			retries += 1;
			if retries >= MAX_RETRIES {
				break;
			} else {
				thread::yield_now();
			}
		}
	}
}

fn try_write(msg_str: &impl Display, output_kind: OutputKind) -> std::io::Result<()> {
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

/// On fail: Inserts error in front of the queue, original message on 2nd spot.
fn blocking_write_first_in_line(msg: Message) {
	let msg_str: &str = msg.as_ref();
	
	if let Err(err) = try_write(&msg_str, msg.output_kind()) {
		let mut queue_guard = QUEUE.lock();
		queue_guard.insert(0, Message::Error(
			format!("comfy_print::blocking_write_first_in_line(): Failed to print first message in queue, it was pushed to the front again.\n\
			Error: {err}\n\
			Message: {msg_str}")));
		queue_guard.insert(1, msg);
	}
}

pub fn _print(input: String) {
	_comfy_print_async(Message::Standard(input));
}

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_print_async(Message::Standard(input));
}

pub fn _eprint(input: String) {
	_comfy_print_async(Message::Error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_print_async(Message::Error(input));
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::async_impl::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::async_impl::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::async_impl::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::async_impl::_eprint(std::format!($($arg)*));
	}};
}

#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::async_impl::_eprintln("\n")
	};
	($($arg:tt)*) => {{
		$crate::async_impl::_eprintln(std::format!($($arg)*));
	}};
}
