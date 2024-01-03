//! Maximum number of successive attempts to print the stored queue.
//! - If [Stdout::write()](std::io::Stdout::write()) or [Stderr::write()](std::io::Stderr::write()) fails, [comfy_print](crate) will attempt to print the queue until [MAX_RETRIES](CURRENT) is reached.
//! - Calling print again will reset the attempts counter.
//! - Does nothing unless [ON_RETRY_PRINTING_FAIL](crate::config::on_queue_printing_fail) == [TryUntilMaxRetries](crate::config::on_queue_printing_fail::On_QueuePrintingFail::TryUntilMaxRetries) (**0**).
//! 
//! # Default: **64**

use std::sync::atomic::{AtomicUsize, Ordering};

/// Current value of [MAX_RETRIES](self).
static CURRENT: AtomicUsize = AtomicUsize::new(64);

/// Environment variable name for global config [MAX_RETRIES](self).
pub const ENV_NAME: &str = "COMFY_PRINT_MAX_RETRIES";

/// Get global config [MAX_RETRIES](self).
pub fn get() -> usize { return CURRENT.load(Ordering::Relaxed); }

/// Set global config [MAX_RETRIES](self).
pub fn set(new_value: usize) { CURRENT.store(new_value, Ordering::Relaxed); }

#[test]
fn test() {
	use crate::test_utils;
	use crate::config;
	
	// Just so the error messages don't interfere with the test.
	config::allow_logging_print_failures::set(false);

	{
		std::env::set_var(ENV_NAME, "10");
		super::env_vars::load_all();
		assert_eq!(get(), 10);
		
		std::env::set_var(ENV_NAME, "5");
		super::env_vars::load_all();
		assert_eq!(get(), 5);
	}

	{
		set(0);
		assert_eq!(get(), 0);

		test_utils::set_toggle_write_fail(true);
		crate::comfy_println!("Test_01");
		assert_eq!(test_utils::get_queue().len(), 1);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 1);
		
		test_utils::set_toggle_write_fail(false);
		crate::comfy_println!("Test_02");

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	{
		set(10);
		assert_eq!(get(), 10);
		
		test_utils::write_fail_once();
		crate::comfy_println!("Test_02");
		assert_eq!(test_utils::get_queue().len(), 1);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}
}
