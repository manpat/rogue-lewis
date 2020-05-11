use std::task::{self, RawWaker, RawWakerVTable, Waker, Poll};
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;

use crate::prelude::*;
use super::promise::*;
use crate::view::{View, ViewCommand};
use crate::gamestate::{GameState, GameCommand};

pub(super) struct CommandQueues {
	pub model_commands: Vec<(GameCommand, UntypedPromise)>,
	pub view_commands: Vec<(ViewCommand, UntypedPromise)>,
}

struct TaskList {
	tasks: Vec<ExecutorTask>,
	task_id: usize,
}

pub struct Executor {
	task_list: Arc<Mutex<TaskList>>,
	wake_queue: Arc<Mutex<Vec<TaskId>>>,

	command_queues: Rc<RefCell<CommandQueues>>,
	hack_gamestate: GameStateHandle,
}

impl Executor {
	pub fn new(hack_gamestate: GameStateHandle) -> Self {
		let command_queues = CommandQueues {
			model_commands: Vec::new(),
			view_commands: Vec::new(),
		};

		let task_list = TaskList {
			tasks: Vec::new(),
			task_id: 0,
		};

		Self {
			task_list: Arc::new(Mutex::new(task_list)),
			wake_queue: Arc::new(Mutex::new(Vec::new())),

			command_queues: Rc::new(RefCell::new(command_queues)),
			hack_gamestate
		}
	}

	pub fn queue<F>(&self, f: F) where F: Future<Output=()> + 'static {
		let mut task_list = self.task_list.lock().unwrap();

		task_list.task_id += 1;
		let task_id = TaskId(task_list.task_id);

		task_list.tasks.push(ExecutorTask{
			future: Box::pin(f),
			task_id,
		});

		self.wake_queue.lock().unwrap().push(task_id);
	}

	pub fn num_queued_tasks(&self) -> usize {
		self.task_list.lock().unwrap().tasks.len()
	}

	pub fn resume_tasks(&self) {
		let wake_queue = std::mem::replace(&mut *self.wake_queue.lock().unwrap(), Vec::new());
		let mut task_list = self.task_list.lock().unwrap();

		for task_id in wake_queue {
			let task = task_list.tasks.iter_mut()
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

		task_list.tasks.retain(|t| t.task_id != INVALID_TASK_ID);
	}

	pub fn process_commands(&self, gamestate: &mut GameState, view: &mut impl View) {
		for (event, promise) in self.command_queues.borrow_mut().model_commands.drain(..) {
			gamestate.submit_command(event, promise);
		}

		for (event, promise) in self.command_queues.borrow_mut().view_commands.drain(..) {
			view.submit_command(event, promise);
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.hack_gamestate.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.hack_gamestate.borrow_mut() }


	pub(super) fn schedule_view_command<O>(&self, cmd: ViewCommand) -> impl Future<Output=O>
		where O: Promisable, Promise<O>: Into<UntypedPromise>
	{
		CommandFuture {
			command_queues: Rc::clone(&self.command_queues),
			state: CommandFutureState::ViewCommand(cmd)
		}
	}

	pub(super) fn schedule_model_command<O>(&self, cmd: GameCommand) -> impl Future<Output=O>
		where O: Promisable, Promise<O>: Into<UntypedPromise>
	{
		CommandFuture {
			command_queues: Rc::clone(&self.command_queues),
			state: CommandFutureState::GameCommand(cmd)
		}
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



enum CommandFutureState<O: Promisable> {
	ViewCommand(ViewCommand),
	GameCommand(GameCommand),
	Future(FutureValue<O>)
}

struct CommandFuture<O: Promisable> {
	command_queues: Rc<RefCell<CommandQueues>>,
	state: CommandFutureState<O>,
}

impl<O: Promisable> Future for CommandFuture<O> where Promise<O>: Into<UntypedPromise> {
	type Output = O;

	fn poll(mut self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		match self.state {
			CommandFutureState::Future(ref mut future) => {
				if future.ready() {
					Poll::Ready(future.unwrap())
				} else {
					Poll::Pending
				}
			}

			CommandFutureState::ViewCommand(cmd) => {
				let promise = O::new_promise(ctx.waker().clone());
				let future = promise.get_future();

				self.command_queues.borrow_mut().view_commands.push((cmd, promise.into()));
				self.state = CommandFutureState::Future(future);

				Poll::Pending
			}

			CommandFutureState::GameCommand(cmd) => {
				let promise = O::new_promise(ctx.waker().clone());
				let future = promise.get_future();

				self.command_queues.borrow_mut().model_commands.push((cmd, promise.into()));
				self.state = CommandFutureState::Future(future);

				Poll::Pending
			}
		}
	}
}
