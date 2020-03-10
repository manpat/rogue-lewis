use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;

#[derive(Debug)]
pub struct BattleController {
	pub boss: bool
	// enemy: monster/boss
}

impl Controller for BattleController {
	fn init(&mut self, _: &GameState) {
		if self.boss {
			println!("Oh fuck it's a boss");
		} else {
			println!("Oh shit it's some monsters");
		}

		println!("Do you fight or flee like a coward?");
	}

	fn run_command(&mut self, _: &mut GameState, command: &str) -> Option<Event> {
		match command {
			"f" | "fight" => {
				println!("You fail");
				None
			}

			"r" | "run" | "flee" => {
				println!("You flee like the coward you are");
				Some(Event::Transition(box MainController))
			}

			_ => {
				println!("the fuck that mean");
				None
			}
		}
	}
}