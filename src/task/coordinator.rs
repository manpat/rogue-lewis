use crate::prelude::*;
use crate::game_state::GameState;
use crate::view::{View, ViewCommand};
use crate::task::{PlayerCommand, ControllerEvent};

use std::task::Waker;

pub(super) struct Inner {
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
		// TODO: pass controller events on to gamestate

		for (event, promise) in self.inner.borrow_mut().unscheduled_view_promises.drain(..) {
			view.submit_command(event, promise);
		}
	}

	pub(super) fn schedule_view_command<O>(&self, cmd: ViewCommand) -> impl Future<Output=O>
		where O: Promisable, Promise<O>: Into<UntypedPromise>
	{
		ViewCommandFuture {
			inner: Rc::clone(&self.inner),
			state: ViewCommandFutureState::Command(cmd)
		}
	}
}





use std::task::Poll;
use std::pin::Pin;


enum ViewCommandFutureState<O: Promisable> {
	Command(ViewCommand),
	Future(FutureValue<O>)
}

struct ViewCommandFuture<O: Promisable> {
	inner: Rc<RefCell<Inner>>,
	state: ViewCommandFutureState<O>,
}

impl<O: Promisable> Future for ViewCommandFuture<O> where Promise<O>: Into<UntypedPromise> {
	type Output = O;

	fn poll(mut self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		match self.state {
			ViewCommandFutureState::Future(ref mut future) => {
				if future.ready() {
					Poll::Ready(future.unwrap())
				} else {
					Poll::Pending
				}
			}

			ViewCommandFutureState::Command(event) => {
				let promise = O::new_promise(ctx.waker().clone());
				let future = promise.get_future();

				self.inner.borrow_mut().unscheduled_view_promises.push((event, promise.into()));
				self.state = ViewCommandFutureState::Future(future);

				Poll::Pending
			}
		}
	}
}


struct PromiseState<T> {
	value: Option<T>,
}

pub struct FutureValue<T> {
	inner: Rc<RefCell<PromiseState<T>>>,
}

impl<T> FutureValue<T> {
	pub fn ready(&self) -> bool { self.inner.borrow().value.is_some() }

	pub fn unwrap(&mut self) -> T {
		self.inner.borrow_mut().value.take()
			.expect("Value not ready yet")
	}
}

pub struct Promise<T> {
	inner: Rc<RefCell<PromiseState<T>>>,
	waker: Waker,
}

impl<T> Promise<T> {
	pub fn new(waker: Waker) -> Promise<T> {
		let state = PromiseState { value: None };

		Promise {
			inner: Rc::new(RefCell::new(state)),
			waker,
		}
	}

	pub fn get_future(&self) -> FutureValue<T> {
		FutureValue {
			inner: Rc::clone(&self.inner)
		}
	}

	pub fn fulfill(self, value: T) {
		let mut inner = self.inner.borrow_mut();
		assert!(inner.value.is_none(), "Promise has already been fulfilled");

		inner.value = Some(value);
		self.waker.wake();
	}
}



pub enum UntypedPromise {
	Void(Promise<()>),
	String(Promise<String>),
	PlayerCommand(Promise<PlayerCommand>),
}

impl UntypedPromise {
	pub fn unwrap_void(self) -> Promise<()> {
		match self {
			UntypedPromise::Void(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to ()")
		}
	}

	pub fn unwrap_string(self) -> Promise<String> {
		match self {
			UntypedPromise::String(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to String")
		}
	}

	pub fn unwrap_player_command(self) -> Promise<PlayerCommand> {
		match self {
			UntypedPromise::PlayerCommand(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to PlayerCommand")
		}
	}
}


pub trait Promisable: Sized {
	fn new_promise(waker: Waker) -> Promise<Self>;
}

impl Promisable for () {
	fn new_promise(waker: Waker) -> Promise<Self> { Promise::new(waker) }
}
impl Promisable for String {
	fn new_promise(waker: Waker) -> Promise<Self> { Promise::new(waker) }
}
impl Promisable for PlayerCommand {
	fn new_promise(waker: Waker) -> Promise<Self> { Promise::new(waker) }
}


impl From<Promise<()>> for UntypedPromise {
	fn from(o: Promise<()>) -> UntypedPromise { UntypedPromise::Void(o) }
}
impl From<Promise<String>> for UntypedPromise {
	fn from(o: Promise<String>) -> UntypedPromise { UntypedPromise::String(o) }
}
impl From<Promise<PlayerCommand>> for UntypedPromise {
	fn from(o: Promise<PlayerCommand>) -> UntypedPromise { UntypedPromise::PlayerCommand(o) }
}