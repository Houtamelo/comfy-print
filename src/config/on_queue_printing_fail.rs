//! Determines what to do if [printing](crate::async_impl::try_write) the queue fails.
//! 0. **TryUntilMaxRetries**: Attempt to print the queue until [MAX_RETRIES](crate::config::max_retries) is reached. 
//! 	- If [MAX_RETRIES](crate::config::max_retries) is reached, this will do whatever [ON_MAX_RETRIES_REACHED](crate::config::on_max_retries_reached) is set to.
//!		- The counter is reset if writing succeeds. 
//! 1. **Return**: do nothing. [comfy_print](crate) will attempt to print the queue next time you use one of the macros.
//! 
//! # Default: [TryUntilMaxRetries](On_QueuePrintingFail::TryUntilMaxRetries)

use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

/// Current value of [ON_QUEUE_PRINTING_FAIL](self).
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// Environment variable name for global config [ON_QUEUE_PRINTING_FAIL](self).
pub const ENV_NAME: &str = "COMFY_PRINT_ON_QUEUE_PRINTING_FAIL";

/// See [ON_QUEUE_PRINTING_FAIL](self).
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum On_QueuePrintingFail {
	/// If printing fails, attempt to print the queue until [MAX_RETRIES](crate::config::max_retries) is reached.
	TryUntilMaxRetries = 0,
	/// If printing fails, do nothing. [comfy_print](crate) will attempt to print the queue next time you use one of the macros.
	Return = 1,
}

impl FromStr for On_QueuePrintingFail {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" | "TryUntilMaxRetries" => Ok(On_QueuePrintingFail::TryUntilMaxRetries),
			"1" | "Return" => Ok(On_QueuePrintingFail::Return),
			_ => Err(format!("Invalid string value for On_RetryPrintingFail: {}", s)),
		}
	}
}

/// Get global config [ON_QUEUE_PRINTING_FAIL](self).
pub fn get() -> On_QueuePrintingFail {
	return match CURRENT.load(Ordering::Relaxed) {
		1 => On_QueuePrintingFail::Return,
		_ => On_QueuePrintingFail::TryUntilMaxRetries, // 0
	};
}

/// Set global config [ON_QUEUE_PRINTING_FAIL](self).
pub fn set(new_value: On_QueuePrintingFail) {
	CURRENT.store(new_value as u8, Ordering::Relaxed);
}

#[test]
fn test() {
	use crate::test_utils;
	use crate::config;

	// Just so the error messages don't interfere with the test.
	config::allow_logging_print_failures::set(false);

	config::max_retries::set(64);

	{
		let current = get();
		std::env::set_var(ENV_NAME, "31413413");
		super::env_vars::load_all();
		assert_eq!(get(), current);
	}

	{
		std::env::set_var(ENV_NAME, "Return");
		super::env_vars::load_all();
		assert_eq!(get(), On_QueuePrintingFail::Return);

		std::env::set_var(ENV_NAME, "TryUntilMaxRetries");
		super::env_vars::load_all();
		assert_eq!(get(), On_QueuePrintingFail::TryUntilMaxRetries);
	}

	{
		set(On_QueuePrintingFail::Return);
		assert_eq!(get(), On_QueuePrintingFail::Return);

		test_utils::write_fail_once();
		crate::comfy_println!("Test_01");
		test_utils::write_fail_once();
		assert_eq!(test_utils::get_queue().len(), 1);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 1);
		
		crate::comfy_println!("Test_02");
		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}

	{
		set(On_QueuePrintingFail::TryUntilMaxRetries);
		assert_eq!(get(), On_QueuePrintingFail::TryUntilMaxRetries);

		test_utils::set_toggle_write_fail(true);
		crate::comfy_println!("Test_01");
		crate::comfy_println!("Test_02");
		crate::comfy_println!("Test_03");
		
		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 3);
		
		test_utils::set_toggle_write_fail(false);
		test_utils::write_fail_once();
		crate::comfy_println!("Test_04");
		assert_eq!(test_utils::get_queue().len(), 4);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
	}
}