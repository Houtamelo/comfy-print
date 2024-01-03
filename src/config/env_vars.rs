//! Functions for loading [comfy_print](crate::config)'s global config variables from the environment.
//! The key for each config is on a const string named `ENV_NAME` in its respective module. Example: [MAX_RETRIES](max_retries::ENV_NAME).
//! Configs are not automatically loaded, you must call [load_all](load_all) to load them. But they do have default values.

use std::env::VarError;
use std::str::FromStr;
use super::*;
use crate::config::on_max_retries_reached::On_MaxRetriesReached;
use crate::config::on_queue_full::On_QueueFull;
use crate::config::on_queue_printing_fail::On_QueuePrintingFail;

/// Errors that can occur when loading a global config variable from the environment.
#[derive(Debug)]
pub enum LoadVarError<T: FromStr> {
	/// Errors returned from [std::env::var](std::env::var).
	VarError(VarError),
	/// Environment variable had a value but it could not be [parsed](FromStr).
	ParseError(T::Err),
	/// Errors returned from [std::fs::File::open](std::fs::File::open).
	IOError(std::io::Error),
}

/// Results for attempting to load each of [comfy_print](crate::config)'s configs from the environment.
pub struct LoadVarsResult {
	/// See [MAX_RETRIES](max_retries).
	pub max_retries: Result<usize, LoadVarError<usize>>,
	/// See [MAX_QUEUE_LENGTH](max_queue_length).
	pub max_queue_length: Result<usize, LoadVarError<usize>>,
	/// See [ALLOW_LOGGING_PRINT_FAILURES](allow_logging_print_failures).
	pub allow_logging_print_failures: Result<bool, LoadVarError<bool>>,
	/// See [ON_RETRY_PRINTING_FAIL](on_queue_printing_fail).
	pub on_retry_printing_fail: Result<On_QueuePrintingFail, LoadVarError<On_QueuePrintingFail>>,
	/// See [ON_MAX_RETRIES_REACHED](on_max_retries_reached).
	pub on_max_retries_reached: Result<On_MaxRetriesReached, LoadVarError<On_MaxRetriesReached>>,
	/// See [LOG_IO_PATH](log_io_path).
	pub log_io_path: Result<String, LoadVarError<String>>,
	/// See [ON_QUEUE_FULL](on_queue_full).
	pub on_push_queue_full: Result<On_QueueFull, LoadVarError<On_QueueFull>>,
}


/// Attempts to load all global config variables from the [environment](std::env). [comfy_print](crate::config)'s global config variables will be replaced by any values found in the environment.
pub fn load_all() -> LoadVarsResult {
	let max_retries = get_var::<usize>(max_retries::ENV_NAME)
			.inspect(|new_value| max_retries::set(*new_value));

	let max_queue_length = get_var::<usize>(max_queue_length::ENV_NAME)
			.inspect(|new_value| max_queue_length::set(*new_value));

	let allow_logging_print_failures = get_var::<bool>(allow_logging_print_failures::ENV_NAME)
			.inspect(|new_value| allow_logging_print_failures::set(*new_value));

	let on_retry_printing_fail = get_var::<On_QueuePrintingFail>(on_queue_printing_fail::ENV_NAME)
			.inspect(|new_value| on_queue_printing_fail::set(*new_value));

	let on_max_retries_reached = get_var::<On_MaxRetriesReached>(on_max_retries_reached::ENV_NAME)
			.inspect(|new_value| on_max_retries_reached::set(*new_value));

	let mut log_io_path: Result<String, LoadVarError<String>> = get_var::<String>(log_io_path::ENV_NAME);

	if let Ok(path) = &mut log_io_path {
		match log_io_path::set(path.as_str()) {
			Ok(_) => {},
			Err(err) => {
				log_io_path = Err(LoadVarError::<String>::IOError(err));
			},
		}
	}

	let on_push_queue_full = get_var::<On_QueueFull>(on_queue_full::ENV_NAME)
			.inspect(|new_value| on_queue_full::set(*new_value));

	return LoadVarsResult {
		max_retries,
		max_queue_length,
		allow_logging_print_failures,
		on_retry_printing_fail,
		on_max_retries_reached,
		log_io_path,
		on_push_queue_full,
	};

	fn get_var<T: FromStr>(var_name: &'static str) -> Result<T, LoadVarError<T>> {
		return match std::env::var(var_name) {
			Ok(value) =>
				match value.parse::<T>() {
					Ok(value) => Ok(value),
					Err(err) => Err(LoadVarError::ParseError(err)),
				},
			Err(err) => Err(LoadVarError::VarError(err)),
		}
	}
}