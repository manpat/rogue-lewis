use std::rc::Rc;
use std::cell::RefCell;
use std::task::Waker;

pub struct Promise<T> {
	inner: Rc<RefCell<SharedState<T>>>,
	waker: Waker,
}

impl<T> Promise<T> {
	pub fn new(waker: Waker) -> Promise<T> {
		let state = SharedState { value: None };

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



pub struct FutureValue<T> {
	inner: Rc<RefCell<SharedState<T>>>,
}

impl<T> FutureValue<T> {
	pub fn ready(&self) -> bool { self.inner.borrow().value.is_some() }

	pub fn unwrap(&mut self) -> T {
		self.inner.borrow_mut().value.take()
			.expect("Value not ready yet")
	}
}


struct SharedState<T> {
	value: Option<T>,
}



use crate::task::PlayerCommand;

pub enum UntypedPromise {
	Void(Promise<()>),
	Bool(Promise<bool>),
	String(Promise<String>),
	PlayerCommand(Promise<PlayerCommand>),
}

impl UntypedPromise {
	pub fn void(self) -> Promise<()> {
		match self {
			UntypedPromise::Void(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to ()")
		}
	}

	pub fn bool(self) -> Promise<bool> {
		match self {
			UntypedPromise::Bool(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to bool")
		}
	}

	pub fn string(self) -> Promise<String> {
		match self {
			UntypedPromise::String(promise) => promise,
			_ => panic!("Failed to unwrap untyped promise to String")
		}
	}

	pub fn player_command(self) -> Promise<PlayerCommand> {
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
impl Promisable for bool {
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
impl From<Promise<bool>> for UntypedPromise {
	fn from(o: Promise<bool>) -> UntypedPromise { UntypedPromise::Bool(o) }
}
impl From<Promise<String>> for UntypedPromise {
	fn from(o: Promise<String>) -> UntypedPromise { UntypedPromise::String(o) }
}
impl From<Promise<PlayerCommand>> for UntypedPromise {
	fn from(o: Promise<PlayerCommand>) -> UntypedPromise { UntypedPromise::PlayerCommand(o) }
}