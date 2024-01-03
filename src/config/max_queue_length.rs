//! Maximum number of messages that can be stored in the queue.
//! - When printing fails, messages will be stored in a shared queue.
//! - [comfy_print](crate) will attempt to print the queue later. See [config::on_retry_printing_fail].
//! - If the queue is full, [ON_QUEUE_FULL](config::on_queue_full) will decide what happens to future messages being pushed in the queue..
//! 
//! # Default: **1024**

use std::sync::atomic::{AtomicUsize, Ordering};

/// Current value of [MAX_QUEUE_LENGTH](self).
static CURRENT: AtomicUsize = AtomicUsize::new(1024);

/// Environment variable name for global config [MAX_QUEUE_LENGTH](self).
pub const ENV_NAME: &str = "COMFY_PRINT_MAX_QUEUE_LENGTH";

/// Get global config [MAX_QUEUE_LENGTH](self).
pub fn get() -> usize { return CURRENT.load(Ordering::Relaxed); }

/// Set global config [MAX_QUEUE_LENGTH](self).
pub fn set(new_value: usize) { CURRENT.store(new_value, Ordering::Relaxed); }

#[test]
fn test() {
	use crate::test_utils;
	use crate::message::OutputKind;
	use crate::config;

	{
		std::env::set_var(ENV_NAME, "20");
		super::env_vars::load_all();
		assert_eq!(get(), 20);
		
		std::env::set_var(ENV_NAME, "5");
		super::env_vars::load_all();
		assert_eq!(get(), 5);
	}

	{
		set(0);
		assert_eq!(get(), 0);

		test_utils::write_fail_once();
		crate::comfy_println!("Test_01");
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	{
		set(1);
		assert_eq!(get(), 1);

		crate::comfy_println!("Test_02");
		assert_eq!(test_utils::get_queue().len(), 0);

		test_utils::write_fail_once();
		crate::comfy_println!("Test_03");

		let queue = test_utils::get_queue();
		assert_eq!(queue.len(), 1);
		assert_eq!(queue[0].output_kind(), OutputKind::Stdout);
		drop(queue);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	{
		config::allow_logging_print_failures::set(true);

		set(10);
		assert_eq!(get(), 10);

		test_utils::set_toggle_write_fail(true);
		for index in 1..=8 {
			crate::comfy_println!("Test_{:02}", index);
		}
		
		test_utils::yield_until_idle();

		let queue = test_utils::get_queue();
		assert_eq!(queue.len(), 10);
		let err_count = queue.iter().filter(|msg| matches!(msg.output_kind(), OutputKind::Stderr)).count();
		assert_eq!(err_count, 2);
		drop(queue);
		
		test_utils::set_toggle_write_fail(false);
		crate::comfy_println!("Test_08");

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	{
		config::allow_logging_print_failures::set(false);

		set(5);
		assert_eq!(get(), 5);
		
		test_utils::set_toggle_write_fail(true);
		for index in 1..=10 {
			crate::comfy_println!("Test_{:02}", index);
		}
		
		let queue = test_utils::get_queue();
		assert_eq!(queue.len(), 5);
		assert!(queue.iter().all(|msg| matches!(msg.output_kind(), OutputKind::Stdout)));
		drop(queue);
		
		test_utils::set_toggle_write_fail(false);
		crate::comfy_eprintln!("Test_10");

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}
}
