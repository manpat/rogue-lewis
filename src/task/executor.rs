use std::task::{self, RawWaker, RawWakerVTable, Waker, Poll};
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;

pub struct Executor {
	tasks: Vec<ExecutorTask>,
	task_id: usize,
	wake_queue: Arc<Mutex<Vec<TaskId>>>,
}

impl Executor {
	pub fn new() -> Self {
		Self {
			tasks: Vec::new(),
			task_id: 0,
			wake_queue: Arc::new(Mutex::new(Vec::new())),
		}
	}

	pub fn queue<F>(&mut self, f: F) where F: Future<Output=()> + 'static {
		self.task_id += 1;
		let task_id = TaskId(self.task_id);

		self.tasks.push(ExecutorTask{
			future: Box::pin(f),
			task_id,
		});

		self.wake_queue.lock().unwrap().push(task_id);
	}

	pub fn num_queued_tasks(&self) -> usize {
		self.tasks.len()
	}

	pub fn has_work_pending(&self) -> bool {
		!self.wake_queue.lock().unwrap().is_empty()
	}

	pub fn poll(&mut self) {
		let wake_queue = std::mem::replace(&mut *self.wake_queue.lock().unwrap(), Vec::new());

		for task_id in wake_queue {
			let task = self.tasks.iter_mut()
				.find(|t| t.task_id == task_id)
				.expect("Tried to wake nonexistant task id");

			let task_state = box TaskState {
				wake_queue: self.wake_queue.clone(),
				task_id: task.task_id
			};

			let raw_waker = new_task_raw_waker(task_state);
			let waker = unsafe { Waker::from_raw(raw_waker) };

			match task.future.as_mut().poll(&mut task::Context::from_waker(&waker)) {
				Poll::Ready(_) => {
					task.task_id = INVALID_TASK_ID;
				}

				Poll::Pending => {}
			}
		}

		self.tasks.retain(|t| t.task_id != INVALID_TASK_ID);
	}
}


#[derive(Debug, Copy, Clone, PartialEq)]
struct TaskId(usize);

const INVALID_TASK_ID: TaskId = TaskId(0);


#[derive(Debug, Clone)]
struct TaskState {
	wake_queue: Arc<Mutex<Vec<TaskId>>>,
	task_id: TaskId,
}


struct ExecutorTask {
	future: Pin<Box<dyn Future<Output=()>>>,
	task_id: TaskId,
}


fn new_task_raw_waker(task: Box<TaskState>) -> RawWaker {
	unsafe fn clone(data: *const ()) -> RawWaker {
		let task_state = &*(data as *const TaskState);
		let new_task_state = Box::new(task_state.clone());

		new_task_raw_waker(new_task_state)
	}

	unsafe fn wake(data: *const ()) {
		let task_state = data as *mut TaskState;
		let task_state = Box::from_raw(task_state);

		task_state.wake_queue.lock()
			.expect("Failed to lock wake queue")
			.push(task_state.task_id);
	}

	unsafe fn wake_by_ref(data: *const ()) {
		let task_state = data as *const TaskState;
		let task_state = &*task_state;

		task_state.wake_queue.lock()
			.expect("Failed to lock wake queue")
			.push(task_state.task_id);
	}

	unsafe fn drop(data: *const ()) {
		let task_state = data as *mut TaskState;
		Box::from_raw(task_state);
	}


	let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
	RawWaker::new(Box::into_raw(task) as *const _, vtable)
}
