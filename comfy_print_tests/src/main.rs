#![feature(test)]

extern crate test;

use std::ops::Range;

use rand::Rng;
use comfy_print::{comfy_eprintln, comfy_println};

use comfy_print::utils::OutputKind;
use libc;

// This has to be a standalone exec (instead of just a tests/mod.rs) because of a bug in printing related to cargo test.
fn main() {
	const COUNT: usize = 500;

	//unsafe { break_stdout();}

	let mut rng = rand::thread_rng();
	
	let bools: Vec<bool> = (0..COUNT).map(|_| rng.gen_bool(0.5)).collect();
	let messages: Vec<String> = generate_messages(0..COUNT).collect();
	print_many_comfy(&messages, &bools);
}

unsafe fn break_stdout() {
	let mut pipe_fds = [0; 2];
	assert_eq!(libc::pipe(&mut pipe_fds as *mut _, 0, 0), 0);
	assert_eq!(libc::close(pipe_fds[0]), 0);
	assert_ne!(libc::dup2(pipe_fds[1], 1), -1);
}

#[bench]
fn benchmarks(bencher: &mut test::Bencher) {
	const COUNT: usize = 500;
	
	let mut rng = rand::thread_rng();
	
	bencher.iter(|| {
		let bools: Vec<bool> = (0..COUNT).map(|_| rng.gen_bool(0.5)).collect();
		let messages: Vec<String> = generate_messages(0..COUNT).collect();
		print_many_std(&messages, &bools);
	});
}

#[allow(unused_must_use)]
fn print_many_std(messages: &[String], bools: &[bool]) {
	std::thread::scope(|s| {
		for _ in 0..10 {
			std::thread::Builder::new().spawn_scoped(s, || {
				for (index, msg_string) in messages.iter().enumerate() {
					match bools[index] {
						true => {
							std::thread::Builder::new()
									.spawn_scoped(s, move || {
										println!("{}", msg_string);
									});
						}
						false => {
							println!("{}", msg_string);
						}
					}
				}
			});
		}
	});
}

#[allow(unused_must_use)]
fn print_many_comfy(messages: &[String], bools: &[bool]) {
	std::thread::scope(|s| {
		for _ in 0..10 {
			std::thread::Builder::new().spawn_scoped(s, || {
				for (index, msg_string) in messages.iter().enumerate() {
					match bools[index] {
						true => {
							std::thread::Builder::new()
									.spawn_scoped(s, move || {
										to_macro_invocation(msg_string, OutputKind::Stderr);
									});
						}
						false => {
							to_macro_invocation(msg_string, OutputKind::Stdout);
						}
					}
				}
			});
		}
	});
}

fn generate_messages(range: Range<usize>) -> impl Iterator<Item = String> {
	let mut rng = rand::thread_rng();
	
	return range.into_iter().map(move |n| {
		if rng.gen_bool(0.5) {
			format!("{n}")
		} else {
			format!("{n}")
		}
	});
}

fn to_macro_invocation(msg_string: &String, output_kind: OutputKind) {
	match output_kind {
		OutputKind::Stdout => {
			comfy_println!("{}", msg_string);
		}
		OutputKind::Stderr => {
			comfy_eprintln!("{}", msg_string);
		}
	}
}
