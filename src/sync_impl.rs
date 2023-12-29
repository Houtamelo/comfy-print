use super::utils;
use super::utils::Message;
use parking_lot::FairMutex;

static QUEUE: FairMutex<Vec<Message>> = FairMutex::new(Vec::new());
use std::sync::atomic::{AtomicBool, Ordering};

static IS_PRINTING: AtomicBool = AtomicBool::new(false);

#[allow(unused_must_use)]
pub fn _comfy_print_sync(msg: Message) {
	let mut queue_guard = QUEUE.lock();
	
	if queue_guard.len() == 0 {
		drop(queue_guard);
		write_first_in_line(msg);
	} else {
		queue_guard.push(msg);
		drop(queue_guard);
		if let Ok(_) = IS_PRINTING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed) {
			write_until_empty();
		}
	}
}

fn write_until_empty() {
	loop {
		let mut queue_guard = QUEUE.lock();
		
		if queue_guard.len() == 0 {
			drop(queue_guard);
			break;
		}
		
		let msg = queue_guard.remove(0);
		drop(queue_guard);
		let msg_str: &str = msg.str();
		let output_kind = msg.output_kind();

		let write_result = utils::try_write(&msg_str, output_kind);
		
		if let Err(err) = write_result {
			let mut queue_guard = QUEUE.lock();
			queue_guard.insert(0, Message::error(format!(
				"comfy_print::write_until_empty(): Failed to print first message in queue, it was pushed to the front again.\n\
				Error: {err}\n\
				Message: {msg_str}\n\
				Target output: {output_kind:?}")));

			queue_guard.insert(1, msg);
			drop(queue_guard);
			break;
		}
	}
	
	IS_PRINTING.store(false, Ordering::Relaxed); // signal other threads that we are no longer printing.
}

/// On fail: Inserts error in front of the queue, original message on 2nd spot.
fn write_first_in_line(msg: Message) {
	let msg_str: &str = msg.str();
	
	if let Err(err) = utils::try_write(&msg_str, msg.output_kind()) {
		let mut queue_guard = QUEUE.lock();
		queue_guard.insert(0, Message::error(format!(
			"comfy_print::blocking_write_first_in_line(): Failed to print first message in queue, it was pushed to the front again.\n\
			Error: {err}\n\
			Message: {msg_str}")));
		
		queue_guard.insert(1, msg);
		drop(queue_guard);
	}
}

pub fn _println(mut input: String) {
	input.push('\n');
	_comfy_print_sync(Message::standard(input));
}

pub fn _print(input: String) {
	_comfy_print_sync(Message::standard(input));
}

pub fn _eprint(input: String) {
	_comfy_print_sync(Message::error(input));
}

pub fn _eprintln(mut input: String) {
	input.push('\n');
	_comfy_print_sync(Message::error(input));
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
