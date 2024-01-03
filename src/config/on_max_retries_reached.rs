//! Determines what to do if [MAX_RETRIES](crate::config::max_retries) is reached.
//! 0. **Return**: do nothing.
//! 1. **WriteToDisk**: Attempt to write stored messages to path specified by [LOG_IO_PATH](crate::config::log_io_path).
//! 	- Any messages successfully written to disk will be removed from the queue.
//! 
//! # Default: [Return](On_MaxRetriesReached::Return)

use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

/// Current value of [ON_MAX_RETRIES_REACHED](self).
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// Environment variable name for global config [ON_MAX_RETRIES_REACHED](self).
pub const ENV_NAME: &str = "COMFY_PRINT_ON_MAX_RETRIES_REACHED";

/// See [ON_MAX_RETRIES_REACHED](self).
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum On_MaxRetriesReached {
	/// If [MAX_RETRIES](crate::config::max_retries) is reached, do nothing.
	Return = 0,
	/// If [MAX_RETRIES](crate::config::max_retries) is reached, attempt to write stored messages to disk.
	WriteToDisk = 1,
}

impl FromStr for On_MaxRetriesReached {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" | "Return" => Ok(On_MaxRetriesReached::Return),
			"1" | "WriteToDisk" => Ok(On_MaxRetriesReached::WriteToDisk),
			_ => Err(format!("Invalid string value for On_MaxRetriesReached: {}", s)),
		}
	}
}

/// Get global config [ON_MAX_RETRIES_REACHED](self).
pub fn get() -> On_MaxRetriesReached {
	return match CURRENT.load(Ordering::Relaxed) {
		1 => On_MaxRetriesReached::WriteToDisk,
		_ => On_MaxRetriesReached::Return, // 0
	};
}

/// Set global config [ON_MAX_RETRIES_REACHED](self).
pub fn set(new_value: On_MaxRetriesReached) {
	CURRENT.store(new_value as u8, Ordering::Relaxed);
}

#[test]
fn test() {
	use crate::test_utils;
	use crate::config;
	
	// Just so the error messages don't interfere with the test.
	config::allow_logging_print_failures::set(false);
	
	// Force max_retries to 0 so we can test the On_MaxRetriesReached branch.
	config::max_retries::set(0);

	{
		let current = get();
		std::env::set_var(ENV_NAME, "123154464");
		super::env_vars::load_all();
		assert_eq!(get(), current);
	}

	{
		std::env::set_var(ENV_NAME, "WriteToDisk");
		super::env_vars::load_all();
		assert_eq!(get(), On_MaxRetriesReached::WriteToDisk);
		
		std::env::set_var(ENV_NAME, "Return");
		super::env_vars::load_all();
		assert_eq!(get(), On_MaxRetriesReached::Return);
	}

	{
		std::env::set_var(ENV_NAME, "0");
		super::env_vars::load_all();
		assert_eq!(get(), On_MaxRetriesReached::Return);
		
		std::env::set_var(ENV_NAME, "1");
		super::env_vars::load_all();
		assert_eq!(get(), On_MaxRetriesReached::WriteToDisk);
	}

	{
		set(On_MaxRetriesReached::Return);
		assert_eq!(get(), On_MaxRetriesReached::Return);

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
}

/// This tests requires Administrator privileges to run.
#[test]
fn test_file_write() {
	use crate::test_utils;
	use crate::config;
	
	// Just so the error messages don't interfere with the test.
	config::allow_logging_print_failures::set(false);

	{
		let mut path = std::env::current_dir().unwrap();
		path.push("/test_log.txt");
		let path = path.to_str().unwrap();

		config::log_io_path::set(path).unwrap();
		let mut fill_me = String::new();
		config::log_io_path::get(&mut fill_me);
		assert_eq!(fill_me.as_str(), path);

		let file = config::log_io_path::get_file().unwrap();
		drop(file);

		set(On_MaxRetriesReached::WriteToDisk);
		assert_eq!(get(), On_MaxRetriesReached::WriteToDisk);

		test_utils::set_toggle_write_fail(true);
		crate::comfy_println!("Test_02");
		assert_eq!(test_utils::get_queue().len(), 1);

		test_utils::yield_until_idle();
		assert_eq!(test_utils::get_queue().len(), 0);
		
		std::fs::remove_file(path).unwrap();
	}
}
