use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;

#[derive(Debug)]
pub struct MerchantController {
	// state: main menu/sell/buy
	// merchant
}

impl Controller for MerchantController {
	fn enter(&mut self, _: &mut GameState) {
		println!("The merchant greets you");
	}

	fn run_command(&mut self, _: &mut GameState, command: &str) -> Option<Event> {
		match command {
			_ => {
				println!("The merchant is insulted by whatever it is you just said and forces you to leave");
				Some(Event::Leave)
			}
		}

	}
}