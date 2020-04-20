use crate::prelude::*;
use crate::game_state::GameState;
use crate::view::{View, ViewEvent, ViewEventType};
use crate::task::{PlayerCommand, ControllerEvent};

use std::task::Waker;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub(super) enum WakeEvent {
	GetPlayerCommand,
	ShowMap { whole_map: bool },
	ControllerEvent(ControllerEvent),
}

pub(super) struct Inner {
	pub scheduled_awaits: Vec<(WakeEvent, Waker)>,
	pub player_command: Option<PlayerCommand>,

	pub scheduled_view_wakeups: HashMap<ViewEventType, Waker>,
}

#[derive(Clone)]
pub struct Coordinator {
	inner: Rc<RefCell<Inner>>,
	hack_game_state: GameStateHandle,
}

impl Coordinator {
	pub fn new(hack_game_state: GameStateHandle) -> Coordinator {
		let inner = Inner {
			scheduled_awaits: Vec::new(),
			player_command: None,

			scheduled_view_wakeups: HashMap::new(),
		};

		Coordinator {
			inner: Rc::new(RefCell::new(inner)),
			hack_game_state
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.hack_game_state.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.hack_game_state.borrow_mut() }

	pub fn run(&mut self, game_state: &mut GameState, view: &mut View) {
		let scheduled_awaits = std::mem::replace(&mut self.inner.borrow_mut().scheduled_awaits, Vec::new());

		for (event, waker) in scheduled_awaits {
			match event {
				WakeEvent::GetPlayerCommand => {
					self.inner.borrow_mut().scheduled_view_wakeups.insert(ViewEventType::PlayerCommand, waker);
					view.request_player_command();
				}

				WakeEvent::ShowMap { whole_map } => {
					self.inner.borrow_mut().scheduled_view_wakeups.insert(ViewEventType::MapShown, waker);
					view.request_show_map(whole_map);
				}

				WakeEvent::ControllerEvent(event) => {
					game_state.notify_event(event);
					view.notify_controller_event(event);

					// TODO: waker should be woken after view is done processing it
					waker.wake();
				}
			}
		}
	}

	pub fn notify_view_event(&self, event: ViewEvent) {
		let mut inner = self.inner.borrow_mut();
		let event_ty = event.to_type();

		match event {
			ViewEvent::PlayerCommand(cmd) => {
				inner.player_command = Some(cmd);
			}

			_ => {}
		}

		if let Some(waker) = inner.scheduled_view_wakeups.remove(&event_ty) {
			waker.wake();
		}
	}

	pub(super) fn notify_controller_event(&self, event: ControllerEvent) -> impl Future<Output=()> {
		self.schedule_event_await(WakeEvent::ControllerEvent(event))
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