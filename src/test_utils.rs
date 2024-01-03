use std::sync::atomic::Ordering;
use parking_lot::lock_api::MutexGuard;
use parking_lot::RawFairMutex;
use crate::async_impl;
use crate::message::Message;

/*/// This is for testing only, there's no unsafe code in the crate.
pub(crate) fn break_stdout() {
	unsafe {
		let mut pipe_fds = [0; 2];
		assert_eq!(libc::pipe(&mut pipe_fds as *mut _, 0, 0), 0);
		assert_eq!(libc::close(pipe_fds[0]), 0);
		assert_ne!(libc::dup2(pipe_fds[1], 1), -1);
	}
}*/

pub(crate) fn get_queue() -> MutexGuard<'static, RawFairMutex, Vec<Message>> {
	return async_impl::QUEUE.lock();
}

pub(crate) fn write_fail_once() {
	async_impl::tests::FORCE_WRITE_FAIL.store(true, Ordering::Relaxed);
}

pub(crate) fn set_toggle_write_fail(value: bool) {
	async_impl::tests::TOGGLE_WRITE_FAIL.store(value, Ordering::Relaxed);
}

pub(crate) fn yield_until_idle() {
	while async_impl::STATE.lock().is_busy() {
		std::thread::yield_now();
	}
}