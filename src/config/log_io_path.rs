//! Path to the file where the queue will be written when [ON_MAX_RETRIES_REACHED](crate::config::on_max_retries_reached) is set to [WriteToDisk](crate::config::on_max_retries_reached::On_MaxRetriesReached::WriteToDisk).
//! - If the file already exists, the messages will be appended to it.
//! - If the file doesn't exist, it will be created.
//! - If the directory doesn't exist, it will be created.
//! 
//! # Default: None

use parking_lot::Mutex;

/// Current value of [DISK_LOG_PATH](self).
static CURRENT: Mutex<String> = Mutex::new(String::new());

/// Environment variable name for global config [DISK_LOG_PATH](self).
pub const ENV_NAME: &str = "COMFY_PRINT_DISK_LOG_PATH";

/// The path stored in global config [DISK_LOG_PATH](self) will be appended to parameter `append_in_me`.
pub fn get(append_in_me: &mut String) {
	let guard = CURRENT.lock();
	append_in_me.push_str(guard.as_str());
	drop(guard);
}

/// Set global config [DISK_LOG_PATH](self).
///
/// Path must include file name and extension.
/// 
/// # Returns
/// 
/// * `Ok(())` if the path was successfully set.
/// * `Err(std::io::Error)` if the path was invalid or if the directory couldn't be created.
pub fn set(new_value: &str) -> Result<(), std::io::Error> {
	let full_path = std::path::Path::new(new_value);
	let Some(dir) = full_path.parent()
			else {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!(
					"Invalid path for DISK_LOG_PATH: {new_value}")));
			};

	match dir.try_exists() {
		Ok(true) => {},
		Ok(false) => {
			if let Err(err) = std::fs::create_dir_all(dir) {
				return Err(err);
			}
		},
		Err(err) => return Err(err),
	}

	let mut guard = CURRENT.lock();
	guard.clear();
	guard.push_str(new_value);
	drop(guard);
	return Ok(());
}

pub(crate) fn get_file() -> Result<std::fs::File, std::io::Error> {
	let guard = CURRENT.lock();
	let path = std::path::Path::new(guard.as_str());

	return std::fs::OpenOptions::new()
			.append(true)
			.create(true)
			.open(path);
}

#[test]
fn test() {
	let mut path = String::new();
	
	{
		set("test.txt").unwrap();
		get(&mut path);
		assert_eq!(path, "test.txt");
	}

	{
		std::env::set_var(ENV_NAME, "test.txt");
		super::env_vars::load_all();
		path.clear();
		get(&mut path);
		assert_eq!(path, "test.txt");

		std::env::set_var(ENV_NAME, "test_05.txt");
		super::env_vars::load_all();
		path.clear();
		get(&mut path);
		assert_eq!(path, "test_05.txt");
	}
}