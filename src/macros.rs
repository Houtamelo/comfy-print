//! # Macros to export for the end user. Documentation largely copied from the original.

/// # Replacement for [std::print!], prints to the standard output.
/// 
/// ---
///
/// - Equivalent to the [`comfy_println!`](crate::comfy_println) macro except that a newline is not printed at the end of the message.
/// - Automatically calls [`flush()`](std::io::Stdout::flush()) afterwards to ensure the output is emitted immediately.
/// - Use [`comfy_print!`](crate::comfy_print) only for the primary output of your program. Use [`comfy_eprint!`](crate::comfy_eprint) instead to print error and progress messages.
/// - See [the formatting documentation in `std::fmt`](../std/fmt/index.html) for details of the macro argument syntax.
///
/// ---
///
/// # Does not panic!
/// Unlike the standard library's [`print!`](std::print) macro, this macro will not panic if writing to stdout fails. 
/// 
/// Instead, it will store the failed message in a queue and attempt to print it later.
/// 
/// ---
/// # Performance
/// 
/// The [`comfy_print!`](crate::comfy_print) macro will lock the standard output on each call. 
/// If you call [`comfy_print!`](crate::comfy_print) within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stdout with [`lock()`](std::io::Stdout::lock()):
/// ```
/// use std::io::{stdout, Write};
///
/// let mut lock = stdout().lock();
/// let result = write!(lock, "hello world");
/// ```
/// 
/// ---
/// 
/// # Examples
///
/// ```
/// use comfy_print::comfy_print;
///
/// comfy_print!("this ");
/// comfy_print!("will ");
/// comfy_print!("be ");
/// comfy_print!("on ");
/// comfy_print!("the ");
/// comfy_print!("same ");
/// comfy_print!("line ");
///
/// comfy_print!("this string has a newline, why not choose comfy_print::comfy_println! instead?\n");
///
/// ```
#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::async_impl::comfy_print_async($crate::message::Message::standard(std::format!($($arg)*)))
    }};
}

/// # Replacement for [std::println!], prints to the standard output, with a newline.
///
/// ---
/// 
/// - On all platforms, the newline is the LINE FEED character (`\n`/`U+000A`) alone (no additional CARRIAGE RETURN (`\r`/`U+000D`)).
/// - Automatically calls [`flush()`](std::io::Stdout::flush()) afterwards to ensure the output is emitted immediately.
/// - This macro uses the same syntax as [`format!`](std::format), but writes to the standard output instead. See [`std::fmt`] for more information.
/// - Use [`comfy_println!`](crate::comfy_println) only for the primary output of your program. Use [`comfy_eprintln!`](crate::comfy_eprintln) instead to print error and progress messages.
/// - See [the formatting documentation in `std::fmt`](../std/fmt/index.html) for details of the macro argument syntax.
///
/// ---
///
/// # Does not panic!
/// Unlike the standard library's [`println!`](std::println) macro, this macro will not panic if writing to stdout fails. 
///
/// Instead, it will store the failed message in a queue and attempt to print it later.
///
/// ---
/// 
/// # Performance
///
/// The [`comfy_println!`](crate::comfy_println) macro will lock the standard output on each call. 
/// If you call [`comfy_println!`](crate::comfy_println) within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stdout with [`lock()`](std::io::Stdout::lock()):
/// ```
/// use std::io::{stdout, Write};
///
/// let mut lock = stdout().lock();
/// let result = writeln!(lock, "hello world");
/// ```
///
/// # Examples
///
/// ```
/// use comfy_print::comfy_println;
/// 
/// comfy_println!(); // prints just a newline
/// comfy_println!("hello there!");
/// comfy_println!("format {} arguments", "some");
/// let local_variable = "some";
/// comfy_println!("format {local_variable} arguments");
/// ```
#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::async_impl::comfy_print_async($crate::message::Message::standard_ln(""))
    };
    ($($arg:tt)*) => {{ 
		$crate::async_impl::comfy_print_async($crate::message::Message::standard_ln(std::format!($($arg)*))) 
	}};
}

/// # Replacement for [`std::eprint!`], prints to the standard error output.
///
/// ---
///
/// - Equivalent to the [`comfy_eprintln!`](crate::comfy_eprintln) macro except that a newline is not printed at the end of the message.
/// - Automatically calls [`flush()`](std::io::Stderr::flush()) afterwards to ensure the output is emitted immediately.
/// - Use [`comfy_eprint!`](crate::comfy_eprint) only for error and progress messages. Use [`comfy_print!`](crate::comfy_print) instead for the primary output of your program.
/// - See [the formatting documentation in `std::fmt`](../std/fmt/index.html) for details of the macro argument syntax.
///
/// ---
///
/// # Does not panic!
/// Unlike the standard library's [`eprint!`](std::eprint) macro, this macro will not panic if writing to stderr fails. 
///
/// Instead, it will store the failed error message in a queue and attempt to print it later.
///
/// ---
/// 
/// # Performance
///
/// The [`comfy_eprint!`](crate::comfy_eprint) macro will lock the error output on each call. 
/// If you call [`comfy_eprint!`](crate::comfy_eprint) within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stderr with [`lock()`](std::io::Stderr::lock()):
/// ```
/// use std::io::{stderr, Write};
///
/// let mut lock = stderr().lock();
/// let result = write!(lock, "hello world, this is a bug!");
/// ```
///
/// ---
///
/// # Examples
///
/// ```
/// comfy_print::comfy_eprint!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! comfy_eprint {
	($($arg:tt)*) => {{
		$crate::async_impl::comfy_print_async($crate::message::Message::error(std::format!($($arg)*)))
	}};
}

/// # Replacement for [std::eprintln!], prints to the error output, with a newline.
///
/// ---
///
/// - On all platforms, the newline is the LINE FEED character (`\n`/`U+000A`) alone (no additional CARRIAGE RETURN (`\r`/`U+000D`)).
/// - Automatically calls [`flush()`](std::io::Stderr::flush()) afterwards to ensure the output is emitted immediately.
/// - This macro uses the same syntax as [`format!`](std::format), but writes to the error output instead. See [`std::fmt`] for more information.
/// - Use [`comfy_eprintln!`](crate::comfy_eprintln) only for error and progress messages. Use [`comfy_println!`](crate::comfy_println) instead for the primary output of your program.
/// - See [the formatting documentation in `std::fmt`](../std/fmt/index.html) for details of the macro argument syntax.
///
/// ---
///
/// # Does not panic!
/// Unlike the standard library's [`eprintln!`](std::eprintln) macro, this macro will not panic if writing to stderr fails. 
///
/// Instead, it will store the failed error message in a queue and attempt to print it later.
///
/// ---
///
/// # Performance
///
/// The [`comfy_eprintln!`](crate::comfy_eprintln) macro will lock the error output on each call. 
/// If you call [`comfy_eprintln!`](crate::comfy_eprintln) within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stderr with [`lock()`](std::io::Stderr::lock()):
/// ```
/// use std::io::{stderr, Write};
///
/// let mut lock = stderr().lock();
/// let result = writeln!(lock, "hello world, this is a bug! With a new line.");
/// ```
///
/// # Examples
///
/// ```
/// use comfy_print::comfy_eprintln;
///
/// comfy_eprintln!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! comfy_eprintln {
	() => {
		$crate::async_impl::comfy_print_async($crate::message::Message::error_ln(""))
	};
	($($arg:tt)*) => {{
		$crate::async_impl::comfy_print_async($crate::message::Message::error_ln(std::format!($($arg)*)))
	}};
}