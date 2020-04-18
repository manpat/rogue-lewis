use crate::prelude::*;
use crate::game_state::GameState;

use std::task::Waker;

struct Inner {
	awaiting_events: Vec<(WakeEvent, Waker)>,
	player_command: Option<String>,
}

#[derive(Copy, Clone, Debug)]
enum WakeEvent {
	PlayerCommand,
}

#[derive(Clone)]
pub struct Coordinator {
	inner: Rc<RefCell<Inner>>,
	game_state: GameStateHandle,
}

impl Coordinator {
	pub fn new(game_state: GameStateHandle) -> Coordinator {
		let inner = Inner {
			awaiting_events: Vec::new(),
			player_command: None,
		};

		Coordinator {
			inner: Rc::new(RefCell::new(inner)),
			game_state
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.game_state.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.game_state.borrow_mut() }

	pub fn run(&self, _game_state: &mut GameState) {
		let awaiting_events = std::mem::replace(&mut self.inner.borrow_mut().awaiting_events, Vec::new());

		for (event, waker) in awaiting_events {
			match event {
				WakeEvent::PlayerCommand => {
					self.inner.borrow_mut().player_command = Some(get_player_command_sync());
					waker.wake();
				}
			}
		}
	}

	fn schedule_wake_event<F, O>(&self, event: WakeEvent, try_get_result: F) -> impl Future<Output=O>
		where F: Fn(&mut Inner) -> Option<O>
	{
		WakeEventFuture {
			inner: Rc::clone(&self.inner),
			event, try_get_result
		}
	}

	pub fn get_player_command(&self) -> impl Future<Output=String> {
		self.schedule_wake_event(WakeEvent::PlayerCommand, |inner| inner.player_command.take())
	}
}



fn get_player_command_sync() -> String {
	use std::io::{Write, BufRead};

	loop {
		print!("> ");

		std::io::stdout().flush()
			.expect("Failed to flush");

		let mut command = std::io::stdin().lock()
			.lines().next()
			.expect("EOF")
			.expect("Failed to read stdin");


		if !command.is_empty() {
			command.make_ascii_lowercase();
			break command
		}
	}
}



use std::task::Poll;
use std::pin::Pin;

struct WakeEventFuture<F, O> where F: Fn(&mut Inner) -> Option<O> {
	inner: Rc<RefCell<Inner>>,
	event: WakeEvent,
	try_get_result: F
}

impl<F, O> Future for WakeEventFuture<F, O> where F: Fn(&mut Inner) -> Option<O> {
	type Output = O;

	fn poll(self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		let mut inner = self.inner.borrow_mut();
		if let Some(cmd) = (self.try_get_result)(&mut inner) {
			Poll::Ready(cmd)
		} else {
			inner.awaiting_events.push((self.event, ctx.waker().clone()));
			Poll::Pending
		}
	}
}