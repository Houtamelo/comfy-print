use super::utils::*;
use std::fmt::Display;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::{FairMutex, RawFairMutex};
use parking_lot::lock_api::MutexGuard;

static QUEUE: FairMutex<Vec<Message>> = FairMutex::new(Vec::new());
static IS_PRINTING: AtomicBool = AtomicBool::new(false);

#[allow(unused_must_use)]
pub fn _comfy_print_sync(msg: Message) {
	let mut queue_guard = QUEUE.lock();
	
	if queue_guard.len() == 0 {
		drop(queue_guard);
		blocking_write_first_in_line(msg);
	} else {
		queue_guard.push(msg);
		
		if let Ok(_) = IS_PRINTING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed) {
			write_until_empty(queue_guard);
		}
	}
}

fn write_until_empty(mut queue_guard: MutexGuard<RawFairMutex, Vec<Message>>) {
	let queue = queue_guard.deref_mut();
	
	while queue.len() > 0 {
		let msg = queue.remove(0);
		let msg_str: &str = msg.as_ref();
		let output_kind = msg.output_kind();
		
		if let Err(err) = try_write(&msg_str, output_kind) {
			queue.insert(0, Message::Error(
				format!("comfy_print::write_until_empty(): Failed to print first message in queue, it was pushed to the front again.\n\
				Error: {err}\n\
				Message: {msg_str}\n\
				Target output: {output_kind:?}")));
			
			queue.insert(1, msg);
			break;
		}
	}

	drop(queue_guard);
	IS_PRINTING.store(false, Ordering::Relaxed);
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
		drop(queue_guard);
	}
}

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_print_sync(Message::Standard(input));
}

pub fn _print(input: String) {
	_comfy_print_sync(Message::Standard(input));
}

pub fn _eprint(input: String) {
	_comfy_print_sync(Message::Error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_print_sync(Message::Error(input));
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::sync_impl::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::sync_impl::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::sync_impl::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::sync_impl::_eprint(std::format!($($arg)*));
	}};
}

#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::sync_impl::_eprintln("\n")
	};
	($($arg:tt)*) => {{
		$crate::sync_impl::_eprintln(std::format!($($arg)*));
	}};
}
