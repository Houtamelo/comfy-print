use comfy_print::utils::Message;

#[cfg(not(feature = "async_tokio"))]
fn main() {
	#[cfg(all(feature = "async_std", not(feature = "async_tokio")))] { test_async_std(); }
	#[cfg(all(not(feature = "async_std"), not(feature = "async_tokio")))] { test_sync(); }
}

#[cfg(feature = "async_tokio")]
fn main() {
	let mut runtime = tokio::runtime::Runtime::new().unwrap();
	comfy_print::async_tokio::write_runtime(&mut runtime, Message::Standard("Run: write_runtime\n".to_string()));
	comfy_print::async_tokio::wait_for_runtime_lock(Message::Standard("Run: wait_for_runtime_lock\n".to_string()));

	comfy_print::async_tokio::write_std_thread(Message::Standard("Run: write_std_thread\n".to_string()));
	comfy_print::async_tokio::_comfy_async_tokio(Message::Standard("Run: _comfy_async_tokio\n".to_string()));

	comfy_print::async_tokio::_print("Run: _print\n".to_string());
	comfy_print::async_tokio::_println("Run: _println".to_string());
	comfy_print::async_tokio::_eprint("Run: _eprint\n".to_string());
	comfy_print::async_tokio::_eprintln("Run: _eprintln".to_string());
	
	comfy_print::async_tokio::wait_for_runtime_lock(Message::Standard("Run: 2 - wait_for_runtime_lock\n".to_string()));
	
	runtime.block_on(async {
		tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
	});
}

#[cfg(all(feature = "async_std", not(feature = "async_tokio")))]
fn test_async_std() {
	comfy_print::async_std::write_thread(Message::Standard("Run: write_thread\n".to_string()));
	comfy_print::async_std::_comfy_async_std(Message::Standard("Run: _comfy_async_std\n".to_string()));
	comfy_print::async_std::_print("Run: _print\n".to_string());
	comfy_print::async_std::_println("Run: _println".to_string());
	comfy_print::async_std::_eprint("Run: _eprint\n".to_string());
	comfy_print::async_std::_eprintln("Run: _eprintln".to_string());
}

#[cfg(all(not(feature = "async_std"), not(feature = "async_tokio")))]
fn test_sync() {
	comfy_print::sync::_comfy_sync(Message::Standard("Run: _comfy_sync\n".to_string()));
	comfy_print::sync::_print("Run: _print\n".to_string());
	comfy_print::sync::_println("Run: _println".to_string());
	comfy_print::sync::_eprint("Run: _eprint\n".to_string());
	comfy_print::sync::_eprintln("Run: _eprintln".to_string());
}
