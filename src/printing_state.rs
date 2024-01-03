use std::thread::JoinHandle;

pub(crate) enum PrintingState {
	Idle,
	Threaded(JoinHandle<()>),
	Synchronous,
}

impl PrintingState {
	pub(crate) fn is_busy(&self) -> bool {
		return match self {
			Self::Idle => false,
			Self::Threaded(handle) => handle.is_finished() == false,
			Self::Synchronous => true,
		};
	}
}