use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;

pub struct MerchantController;

impl Controller for MerchantController {
	fn init(&mut self, _: &GameState) {
		println!("The merchant greets you");
	}

	fn run_command(&mut self, _: &mut GameState, command: &str) -> Event {
		match command {
			_ => {
				println!("The merchant is insulted by whatever it is you just said and forces you to leave");
				Event::Transition(box MainController)
			}
		}

	}
}