use crate::prelude::*;
use crate::task::promise::*;
use crate::task::executor::TaskId;
use std::task::Poll;
use std::future::Future;
use std::pin::Pin;

pub struct AnimationQueue<V> {
	inner: Rc<RefCell<AnimationQueueInner<V>>>
}

struct AnimationQueueInner<V> {
	tasks: Vec<(Box<dyn FnOnce(&mut V)>, Promise<()>)>
}


impl<V> AnimationQueue<V> {
	pub fn new() -> Self {
		let inner = AnimationQueueInner {
			tasks: Vec::new()
		};

		AnimationQueue {
			inner: Rc::new(RefCell::new(inner))
		}
	}

	pub fn get(&mut self) -> Vec<(Box<dyn FnOnce(&mut V)>, Promise<()>)> {
		std::mem::replace(&mut self.inner.borrow_mut().tasks, Vec::new())
	}

	pub fn start<C, F>(&mut self, construct_future: C) -> TaskId
		where C: (FnOnce(AnimationContext<V>) -> F) + 'static,
			F: Future<Output=()> + 'static
	{
		let ctx = AnimationContext::new(&self);
		get_executor().queue(construct_future(ctx))
	}
}




pub struct AnimationContext<V> {
	inner: Rc<RefCell<AnimationQueueInner<V>>>,
}

impl<V> AnimationContext<V> {
	pub fn new(view: &AnimationQueue<V>) -> Self {
		Self { inner: Rc::clone(&view.inner) }
	}

	pub fn run<F>(&self, f: F) -> impl Future<Output=()>
		where F: FnOnce(&mut V) + 'static
	{
		AnimationFuture {
			inner: Rc::clone(&self.inner),
			state: AnimationFutureState::Pending(Some(Box::new(f)))
		}
	}
}





enum AnimationFutureState<V> {
	// Option used here bc apparently I can't move the Box otherwise??
	Pending(Option<Box<dyn FnOnce(&mut V) + 'static>>),
	Future(FutureValue<()>)
}

struct AnimationFuture<V> {
	inner: Rc<RefCell<AnimationQueueInner<V>>>,
	state: AnimationFutureState<V>,
}

impl<V> Future for AnimationFuture<V> {
	type Output = ();

	fn poll(mut self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		match self.state {
			AnimationFutureState::Future(ref mut future) => {
				if future.ready() {
					Poll::Ready(future.unwrap())
				} else {
					Poll::Pending
				}
			}

			AnimationFutureState::Pending(ref mut f) => {
				let promise = <()>::new_promise(ctx.waker().clone());
				let future = promise.get_future();

				let f = f.take().unwrap();
				self.inner.borrow_mut().tasks.push((f, promise.into()));
				self.state = AnimationFutureState::Future(future);

				Poll::Pending
			}
		}
	}
}
