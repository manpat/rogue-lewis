use crate::prelude::*;
use crate::game_state::GameState;
use crate::view::{View, ViewCommand};
use crate::task::{PlayerCommand, GameCommand};
use crate::task::promise::*;


pub(super) struct Inner {
	pub unscheduled_model_promises: Vec<(GameCommand, UntypedPromise)>,
	pub unscheduled_view_promises: Vec<(ViewCommand, UntypedPromise)>,
}

#[derive(Clone)]
pub struct Coordinator {
	inner: Rc<RefCell<Inner>>,
	hack_game_state: GameStateHandle,
}

impl Coordinator {
	pub fn new(hack_game_state: GameStateHandle) -> Coordinator {
		let inner = Inner {
			unscheduled_model_promises: Vec::new(),
			unscheduled_view_promises: Vec::new(),
		};

		Coordinator {
			inner: Rc::new(RefCell::new(inner)),
			hack_game_state
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.hack_game_state.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.hack_game_state.borrow_mut() }

	pub fn run(&mut self, game_state: &mut GameState, view: &mut View) {
		for (event, promise) in self.inner.borrow_mut().unscheduled_model_promises.drain(..) {
			game_state.submit_command(event, promise);
		}

		for (event, promise) in self.inner.borrow_mut().unscheduled_view_promises.drain(..) {
			view.submit_command(event, promise);
		}
	}

	pub(super) fn schedule_view_command<O>(&self, cmd: ViewCommand) -> impl Future<Output=O>
		where O: Promisable, Promise<O>: Into<UntypedPromise>
	{
		CommandFuture {
			inner: Rc::clone(&self.inner),
			state: CommandFutureState::ViewCommand(cmd)
		}
	}

	pub(super) fn schedule_model_command<O>(&self, cmd: GameCommand) -> impl Future<Output=O>
		where O: Promisable, Promise<O>: Into<UntypedPromise>
	{
		CommandFuture {
			inner: Rc::clone(&self.inner),
			state: CommandFutureState::GameCommand(cmd)
		}
	}
}





use std::task::Poll;
use std::pin::Pin;


enum CommandFutureState<O: Promisable> {
	ViewCommand(ViewCommand),
	GameCommand(GameCommand),
	Future(FutureValue<O>)
}

struct CommandFuture<O: Promisable> {
	inner: Rc<RefCell<Inner>>,
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

				self.inner.borrow_mut().unscheduled_view_promises.push((cmd, promise.into()));
				self.state = CommandFutureState::Future(future);

				Poll::Pending
			}

			CommandFutureState::GameCommand(cmd) => {
				let promise = O::new_promise(ctx.waker().clone());
				let future = promise.get_future();

				self.inner.borrow_mut().unscheduled_model_promises.push((cmd, promise.into()));
				self.state = CommandFutureState::Future(future);

				Poll::Pending
			}
		}
	}
}
