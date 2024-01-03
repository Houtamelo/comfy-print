use std::io::Write;
use std::thread;
use parking_lot::{FairMutex, RawFairMutex};
use parking_lot::lock_api::MutexGuard;
use config::on_queue_full::On_QueueFull;
use crate::message::{Message, OutputKind};
use crate::printing_state::PrintingState;
use crate::config;
use crate::config::on_max_retries_reached::On_MaxRetriesReached;
use crate::config::on_queue_printing_fail::On_QueuePrintingFail;

/// This is public within crate to allow testing.
pub(crate) static QUEUE: FairMutex<Vec<Message>> = FairMutex::new(Vec::new());

pub(crate) static STATE: FairMutex<PrintingState> = FairMutex::new(PrintingState::Idle);

/// Main function for printing user messages.
/// 
/// # Arguments 
/// 
/// * `msg`: [Message] to be printed.
/// 
/// # Examples 
/// 
/// ```
/// use comfy_print::message::Message;
/// 
/// let msg = Message::standard("Hello, world!");
/// // Prints "Hello, world!" to stdout.
/// comfy_print::async_impl::comfy_print_async(msg);
///
/// let msg = Message::error("Hello, bug!");
/// // Prints "Hello, bug!" to stderr.
/// comfy_print::async_impl::comfy_print_async(msg);
///
/// let msg = Message::standard_ln("Hello, world!");
/// // Prints "Hello, world!\n" to stdout.
/// comfy_print::async_impl::comfy_print_async(msg);
///
/// let msg = Message::error_ln("Hello, bug!");
/// // Prints "Hello, bug!\n" to stderr.
/// comfy_print::async_impl::comfy_print_async(msg);
/// 
/// ```
#[allow(unused_must_use)]
pub fn comfy_print_async(msg: Message) {
	let mut queue_guard = QUEUE.lock();
	let queue_len = queue_guard.len();
	
	if queue_len == 0 {
		drop(queue_guard);
		
		try_write(&msg).inspect_err(
			|err| {
				if config::max_queue_length::get() == 0 {
					return;
				}

				let mut queue_guard = QUEUE.lock();
				queue_guard.insert(0, msg);
				owned_try_insert_write_err(&mut queue_guard, err, "comfy_print::async_impl::comfy_print_async(): Failed to print message, creating queue...");
				drop(queue_guard);
				
				check_state();
			});
	} 
	else {
		if queue_len < config::max_queue_length::get(){
			queue_guard.push(msg);
		} else if On_QueueFull::KeepNewest == config::on_queue_full::get() {
			queue_guard.remove(0);
			queue_guard.push(msg);
		}
		
		drop(queue_guard);
		
		check_state();
	}

	return;
	
	/// WARNING: May lock [STATE], then may lock [QUEUE].
	fn check_state() {
		let Some(mut state_guard) = STATE.try_lock()
				else { return; };

		if state_guard.is_busy() { // We already pushed our msg to the queue and there's already someone else printing it, so we can return.
			drop(state_guard);
			return;
		}

		let thread_result = thread::Builder::new().spawn(start_printing_queue);

		match thread_result {
			Ok(handle) => {
				*state_guard = PrintingState::Threaded(handle);
				drop(state_guard);
			}
			Err(err) => {
				*state_guard = PrintingState::Synchronous;
				drop(state_guard);

				try_insert_write_err(&err, "`comfy_print::async_impl::check_state()`: Failed to create a thread to print the queue.");

				start_printing_queue();

				let mut state_guard = STATE.lock();
				*state_guard = PrintingState::Idle;
				drop(state_guard);
			}
		}
	}
}

fn start_printing_queue() {
	print_until_empty(config::max_retries::get(), 0);
}

/// WARNING: Will lock [QUEUE], then may lock [std::io::stdout] and/or [std::io::stderr].
fn print_until_empty(max_retries: usize, retries: usize) {
	let mut queue_guard = QUEUE.lock();
	
	if queue_guard.is_empty() {
		queue_guard.shrink_to_fit();
		drop(queue_guard);
		return;
	}
	
	let msg = queue_guard.remove(0);
	drop(queue_guard); // unlock the queue before blocking stdout/err
	
	match try_write(&msg) {
		Ok(_) => {
			print_until_empty(max_retries, retries);
		},
		Err(err) => match config::on_queue_printing_fail::get() {
			On_QueuePrintingFail::TryUntilMaxRetries => {
				reinsert_message(msg, err);

				if retries < max_retries {
					print_until_empty(max_retries, retries + 1);
				} else {
					on_max_retries();
				}
			}
			On_QueuePrintingFail::Return => {
				reinsert_message(msg, err);
				return;
			}
		}
	}
	
	return;

	/// WARNING: Will lock [QUEUE].
	fn reinsert_message(msg: Message, err: std::io::Error) {
		let mut queue_guard = QUEUE.lock();

		// This can happen if another thread pushed a message to the queue while we were printing the current one.
		if queue_guard.len() < config::max_queue_length::get() {
			queue_guard.insert(0, msg);
		} else if let On_QueueFull::KeepOldest = config::on_queue_full::get() {
			queue_guard.pop();
			queue_guard.insert(0, msg);
		}

		owned_try_insert_write_err(&mut queue_guard, &err, "`comfy_print::async_impl::print_until_empty()`: Failed to print first message in queue.");
		drop(queue_guard);
	}

	/// WARNING: May lock [QUEUE].
	fn on_max_retries() {
		match config::on_max_retries_reached::get() {
			On_MaxRetriesReached::Return => {
				return;
			},
			On_MaxRetriesReached::WriteToDisk => {
				let Ok(mut file) = config::log_io_path::get_file()
						else { return; };

				let mut queue_guard = QUEUE.lock();

				while !queue_guard.is_empty() {
					let msg = &queue_guard[0];
					let write_result = write!(file, "{}", msg);

					match write_result {
						Ok(_) => {
							queue_guard.remove(0);
							continue;
						},
						Err(err) => {
							owned_try_insert_write_err(&mut queue_guard, &err, "`comfy_print::async_impl::on_max_retries_reached()`: Failed to write to log file.");
							break;
						}
					}
				}

				queue_guard.shrink_to_fit();
				drop(queue_guard);
				drop(file);
			}
		}
	}
}

#[cfg(not(test))]
/// WARNING: Will lock one of [std::io::stdout] | [std::io::stderr]
fn try_write(msg: &Message) -> std::io::Result<()> { 
	match msg.output_kind() {
		OutputKind::Stdout => {
			let mut stdout = std::io::stdout().lock();
			write!(stdout, "{}", msg)?;
			stdout.flush()?;
			Ok(())
		}
		OutputKind::Stderr => {
			let mut stderr = std::io::stderr().lock();
			write!(stderr, "{}", msg)?;
			stderr.flush()?;
			Ok(())
		}
	}
}

#[cfg(test)]
/// WARNING: Will lock one of [std::io::stdout] | [std::io::stderr]
fn try_write(msg: &Message) -> std::io::Result<()> {
	use std::sync::atomic::Ordering;
	
	if tests::TOGGLE_WRITE_FAIL.load(Ordering::Relaxed) == true {
		return Err(std::io::Error::new(std::io::ErrorKind::Other, tests::FORCE_WRITE_FAIL_MSG));
	}
	
	let force_write_fail_result = tests::FORCE_WRITE_FAIL
			.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);

	if let Ok(_) = force_write_fail_result {
		return Err(std::io::Error::new(std::io::ErrorKind::Other, tests::FORCE_WRITE_FAIL_MSG));
	}

	match msg.output_kind() {
		OutputKind::Stdout => {
			let mut stdout = std::io::stdout().lock();
			write!(stdout, "{}", msg)?;
			stdout.flush()?;
			Ok(())
		}
		OutputKind::Stderr => {
			let mut stderr = std::io::stderr().lock();
			write!(stderr, "{}", msg)?;
			stderr.flush()?;
			Ok(())
		}
	}
}

/// WARNING: Will lock [QUEUE]
#[inline(always)]
fn try_insert_write_err(err: &std::io::Error, call_description: &'static str) {
	if config::allow_logging_print_failures::get() == false {
		return;
	}
	
	let max_length = config::max_queue_length::get();
	let mut queue_guard: MutexGuard<RawFairMutex, Vec<Message>> = QUEUE.lock();
	if queue_guard.len() < max_length {
		queue_guard.insert(0, Message::error_ln(format!("{call_description}\nError: {err}.")));
	}
	
	drop(queue_guard);
}

/// WARNING: does not lock anything since this receives a mutable reference to a queue.
#[inline(always)]
fn owned_try_insert_write_err(queue_guard: &mut MutexGuard<RawFairMutex, Vec<Message>>, err: &std::io::Error, call_description: &'static str) {
	if config::allow_logging_print_failures::get() == false {
		return;
	}

	let max_length = config::max_queue_length::get();
	if queue_guard.len() < max_length {
		queue_guard.insert(0, Message::error_ln(format!("{call_description}\nError: {err}.")));
	}
}

#[cfg(test)]
pub(crate) mod tests {
	pub(crate) static FORCE_WRITE_FAIL: AtomicBool = AtomicBool::new(false);
	pub(crate) static TOGGLE_WRITE_FAIL: AtomicBool = AtomicBool::new(false);
	pub const FORCE_WRITE_FAIL_MSG: &str = "Forced write failure";

	use std::sync::atomic::AtomicBool;
	use crate::comfy_println;
	use super::*;
	use crate::test_utils;
	
	#[test]
	fn test_when_queue_is_empty() {
		comfy_println!("Test message");
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	#[test]
	fn test_when_queue_is_not_empty() {
		// Just so the error messages don't interfere with the test.
		config::allow_logging_print_failures::set(false);
		
		test_utils::set_toggle_write_fail(true);
		comfy_println!("Test message_1");
		comfy_println!("Test message_2");
		comfy_println!("Test message_3");
		
		assert_eq!(STATE.lock().is_busy(), true);
		
		test_utils::yield_until_idle();
		assert_eq!(STATE.lock().is_busy(), false);
		assert_eq!(test_utils::get_queue().len(), 3);

		test_utils::set_toggle_write_fail(false);
		comfy_println!("Test message_4");
		assert_eq!(STATE.lock().is_busy(), true);
		
		test_utils::yield_until_idle();
		assert_eq!(STATE.lock().is_busy(), false);
		assert_eq!(test_utils::get_queue().len(), 0);
	}
}