//! # Configuration
//! 
//! At runtime, [comfy_print](crate) allows configuring its behavior through global variables.
//! These variables can be set manually by calling the `set` function of each variable's module. Example: [max_queue_length::set()].
//! You may also read these variables from the environment by calling [env_vars::load_all()].
//! 
//! See each module's documentation for more information.

#![allow(non_camel_case_types)]

pub mod env_vars;

pub mod max_queue_length;
pub mod max_retries;
pub mod allow_logging_print_failures;
pub mod on_queue_full;
pub mod on_max_retries_reached;
pub mod on_queue_printing_fail;
pub mod log_io_path;