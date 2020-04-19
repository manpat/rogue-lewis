use crate::prelude::*;
use crate::game_state::GameState;
use crate::view::View;

use std::task::Waker;

pub(super) struct Inner {
	pub scheduled_awaits: Vec<(WakeEvent, Waker)>,
	pub player_command: Option<String>,

	on_player_command: Option<Waker>,
	on_map_shown: Option<Waker>,
}

#[derive(Copy, Clone, Debug)]
pub(super) enum WakeEvent {
	GetPlayerCommand,
	ShowMap { whole_map: bool },
}

#[derive(Clone)]
pub struct Coordinator {
	inner: Rc<RefCell<Inner>>,
	game_state: GameStateHandle,
}

impl Coordinator {
	pub fn new(game_state: GameStateHandle) -> Coordinator {
		let inner = Inner {
			scheduled_awaits: Vec::new(),
			player_command: None,

			on_player_command: None,
			on_map_shown: None,
		};

		Coordinator {
			inner: Rc::new(RefCell::new(inner)),
			game_state
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.game_state.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.game_state.borrow_mut() }

	pub fn run(&mut self, _game_state: &mut GameState, view: &mut View) {
		let scheduled_awaits = std::mem::replace(&mut self.inner.borrow_mut().scheduled_awaits, Vec::new());

		for (event, waker) in scheduled_awaits {
			match event {
				WakeEvent::GetPlayerCommand => {
					self.inner.borrow_mut().on_player_command = Some(waker);
					view.request_player_command();
				}

				WakeEvent::ShowMap { whole_map } => {
					// TODO: maybe the view should just emit an event instead of taking the waker directly?
					self.inner.borrow_mut().on_map_shown = Some(waker);
					view.request_show_map(whole_map);
				}
			}
		}
	}

	pub fn notify_map_shown(&self) {
		if let Some(waker) = self.inner.borrow_mut().on_map_shown.take() {
			waker.wake();
		}
	}

	pub fn notify_player_command(&self, cmd: String) {
		self.inner.borrow_mut().player_command = Some(cmd);

		if let Some(waker) = self.inner.borrow_mut().on_player_command.take() {
			waker.wake();
		}
	}

	pub(super) fn schedule_value_await<F, O>(&self, event: WakeEvent, try_get_result: F) -> impl Future<Output=O>
		where F: Fn(&mut Inner) -> Option<O>
	{
		WakeEventCallbackFuture {
			inner: Rc::clone(&self.inner),
			event, try_get_result
		}
	}

	pub(super) fn schedule_event_await(&self, event: WakeEvent) -> impl Future<Output=()>
	{
		WakeEventFuture {
			inner: Rc::clone(&self.inner),
			event,
			scheduled: false
		}
	}
}





use std::task::Poll;
use std::pin::Pin;

struct WakeEventCallbackFuture<F, O> where F: Fn(&mut Inner) -> Option<O> {
	inner: Rc<RefCell<Inner>>,
	event: WakeEvent,
	try_get_result: F
}

impl<F, O> Future for WakeEventCallbackFuture<F, O> where F: Fn(&mut Inner) -> Option<O> {
	type Output = O;

	fn poll(self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		let mut inner = self.inner.borrow_mut();
		if let Some(cmd) = (self.try_get_result)(&mut inner) {
			Poll::Ready(cmd)
		} else {
			inner.scheduled_awaits.push((self.event, ctx.waker().clone()));
			Poll::Pending
		}
	}
}

struct WakeEventFuture {
	inner: Rc<RefCell<Inner>>,
	event: WakeEvent,
	scheduled: bool
}

impl Future for WakeEventFuture {
	type Output = ();

	fn poll(mut self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		if self.scheduled {
			Poll::Ready(())
		} else {
			self.scheduled = true;
			self.inner.borrow_mut().scheduled_awaits.push((self.event, ctx.waker().clone()));
			Poll::Pending
		}
	}
}