use crate::prelude::*;
use crate::game_state::GameState;

struct Inner {

}

#[derive(Clone)]
pub struct Coordinator {
	inner: Rc<RefCell<Inner>>,
	game_state: GameStateHandle,
}

impl Coordinator {
	pub fn new(game_state: GameStateHandle) -> Coordinator {
		Coordinator {
			inner: Rc::new(RefCell::new(Inner {})),
			game_state
		}
	}

	pub fn hack_game(&self) -> std::cell::Ref<GameState> { self.game_state.borrow() }
	pub fn hack_game_mut(&self) -> std::cell::RefMut<GameState> { self.game_state.borrow_mut() }

	pub fn get_player_command(&self) -> impl Future<Output=String> {
		async {
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
	}
}