use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;

pub struct BattleController;

impl Controller for BattleController {
	fn init(&mut self, _: &GameState) {
		println!("The battle begins");
	}

	fn run_command(&mut self, _: &mut GameState, command: &str) -> Event {
		match command {
			"f" | "fight" => {
				println!("You fail");
				Event::Continue
			}

			"r" | "run" | "flee" => {
				println!("You flee like the coward you are");
				Event::Transition(box MainController)
			}

			_ => {
				println!("the fuck that mean");
				Event::Continue
			}
		}
	}
}