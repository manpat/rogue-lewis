use crate::prelude::*;
use crate::controller::*;
use crate::game_state::*;
use crate::enemy::*;

#[derive(Debug)]
pub struct BattleController {
	location: Location,
	enemy: Option<Enemy>,
}

impl Controller for BattleController {
	fn enter(&mut self, state: &mut GameState) {
		let enemy = state.get_enemy(self.location)
			.expect("Tried to start battle with no enemy");

		self.enemy = Some(enemy);

		if enemy.archetype.is_boss() {
			println!("Oh fuck it's a boss");
		} else {
			println!("Oh shit it's some monsters");
		}

		println!("Do you fight or run like a coward?");
	}

	fn leave(&mut self, state: &mut GameState) {
		if let Some(enemy) = self.enemy {
			if enemy.health > 0 {
				state.update_enemy(self.location, enemy);
			} else {
				state.remove_encounter_at(self.location);
			}
		} else {
			state.remove_encounter_at(self.location);	
		}
	}

	fn run_command(&mut self, state: &mut GameState, command: &str) -> Option<Event> {
		match command {
			"f" | "fight" => {
				println!("You fail");
				None
			}

			"e" | "eat" | "h" | "heal" => {
				if state.player.inventory.take(Item::Food) {
					let health_gain: i32 = rng().gen_range(1, 4);
					state.player.health += health_gain;
					println!("You recover {} health", health_gain);
				} else {
					println!("You don't have enough food!");
				}
			}

			"r" | "run" | "flee" => {
				println!("You flee like the coward you are");
				Some(Event::Leave)
			}

			_ => {
				println!("the fuck that mean");
				None
			}
		}
	}
}

impl BattleController {
	pub fn new(location: Location) -> BattleController {
		BattleController { location, enemy: None }
	}
}