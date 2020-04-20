use crate::prelude::*;
use crate::controller::*;
use crate::game_state::*;
use crate::enemy::*;
use crate::task;

#[derive(Debug)]
pub struct BattleController {
	location: Location,
	enemy: Option<Enemy>,
}


#[derive(Debug, Copy, Clone)]
enum AttackSeverity {
	Crit,
	Hit,
	Miss,
}


impl BattleController {
	pub fn new(location: Location) -> BattleController {
		BattleController { location, enemy: None }
	}

	fn enemy_archetype(&self) -> EnemyArchetype { self.enemy.unwrap().archetype }
	fn is_enemy_boss(&self) -> bool { self.enemy_archetype().is_boss() }

	fn run_player_attack(&mut self) -> Option<Event> {
		use std::cmp::Ordering;
		use AttackSeverity::*;

		let mut r = rng();

		let enemy_max_roll = if self.is_enemy_boss() { 13 } else { 10 };

		let player_roll = r.gen_range(0, 10);
		let enemy_roll = r.gen_range(0, enemy_max_roll);

		match player_roll.cmp(&enemy_roll) {
			Ordering::Greater => {
				let severity = match player_roll {
					1 => Miss,
					9..=10 => Crit,
					_ => Hit,
				};

				self.apply_player_attack(severity)
			}

			Ordering::Equal => {
				println!("Your weapons clash and neither you nor the {:?} take damage", self.enemy_archetype());
				None
			}

			Ordering::Less => {
				let severity = if self.is_enemy_boss() {
					match enemy_roll {
						1 => Miss,
						11..=13 => Crit,
						_ => Hit,
					}
				} else {
					match enemy_roll {
						1 => Miss,
						9..=10 => Crit,
						_ => Hit,
					}
				};

				self.apply_enemy_attack(severity, false)
			}
		}

	}

	fn run_enemy_attack_during_heal(&self) -> Option<Event> {
		use AttackSeverity::*;

		let probabilities = if self.is_enemy_boss() {[2, 2, 6]} else {[3, 4, 6]};
		let severity = choose_with_weights(&[Crit, Hit, Miss], &probabilities);

		self.apply_enemy_attack(severity, false)
	}

	fn run_enemy_attack_during_flee(&self) -> Option<Event> {
		if rng().gen_ratio(2, 5) {
			self.apply_enemy_attack(AttackSeverity::Hit, true)
		} else {
			None
		}
	}


	fn apply_player_attack(&mut self, severity: AttackSeverity) -> Option<Event> {
		let enemy_archetype = self.enemy_archetype();
		let mut damage = get_coordinator().hack_game().player.attack();

		match severity {
			AttackSeverity::Crit => {
				println!("You strike the {:?} and it takes critical damage!", enemy_archetype);
				damage *= 2;
			}

			AttackSeverity::Hit => {
				println!("You strike the {:?}!", enemy_archetype);
			}

			AttackSeverity::Miss => {
				println!("You strike the {:?} but you miss", enemy_archetype);
				return None
			}
		}

		let enemy_defense = enemy_archetype.defense();
		let shield_applied = enemy_defense > 0 && rng().gen_ratio(3, 4);

		if shield_applied {
			println!("The {:?} raises it's shield and blocks some of your attack", enemy_archetype);
			damage -= enemy_defense;
		}

		let enemy = self.enemy.as_mut().unwrap();
		enemy.health -= damage.max(0);

		if enemy.is_dead() {
			println!("The strike is fatal! The {:?} is defeated!", enemy_archetype);
			Some(Event::Leave)
		} else {
			None
		}
	}


	fn apply_enemy_attack(&self, severity: AttackSeverity, ignore_shield: bool) -> Option<Event> {
		let enemy_archetype = self.enemy_archetype();
		let coordinator = get_coordinator();
		let mut game_state = coordinator.hack_game_mut();
		let player = &mut game_state.player;
		let mut damage = enemy_archetype.attack();

		match severity {
			AttackSeverity::Crit => {
				println!("The {:?} strikes you and you take critical damage!", enemy_archetype);
				damage *= 2;
			}

			AttackSeverity::Hit => {
				println!("The {:?} strikes you!", enemy_archetype);
			}

			AttackSeverity::Miss => {
				println!("The {:?} swings at you but misses", enemy_archetype);
				return None
			}
		}

		let player_defense = player.defense();
		let shield_applied = player_defense > 0 && rng().gen_ratio(3, 4);

		if shield_applied && !ignore_shield {
			println!("You raise your shield in time to take some of the blow");
			damage -= player_defense;
		}

		player.health -= damage.max(0);

		if player.is_dead() {
			println!("Unfortunately, the strike is fatal");
			Some(Event::Lose)
		} else {
			None
		}
	}
}


pub async fn run_battle_controller(loc: Location) {
	println!("[battle] enter");

	let ctx = get_coordinator().clone();
	let mut ctl = BattleController::new(loc);

	let enemy = ctx.hack_game().get_enemy(loc)
		.expect("Tried to start battle with no enemy");

	ctl.enemy = Some(enemy);

	if enemy.archetype.is_boss() {
		println!("Oh fuck it's a boss!");
		println!("The {:?} sets its eyes on you", enemy.archetype);
	} else {
		println!("Oh shit it's a monster");
		println!("The {:?} readies its weapon", enemy.archetype);
	}

	println!("Do you fight or run like a coward?");

	loop {
		let command = task::get_player_command().await;

		match command.0.as_str() {
			"f" | "fight" => match ctl.run_player_attack() {
				Some(Event::Lose) => {
					println!("lose??");
					break
				}

				Some(Event::Leave) => { break }

				_ => {}
			}

			"e" | "eat" | "h" | "heal" => {
				if ctx.hack_game_mut().player.inventory.take(Item::Food) {
					let health_gain: i32 = rng().gen_range(1, 4);
					ctx.hack_game_mut().player.health += health_gain;
					println!("You recover {} health", health_gain);

					if let Some(Event::Lose) = ctl.run_enemy_attack_during_heal() {
						println!("Lose ???");
						break;
					}

				} else {
					println!("You don't have enough food!");
				}
			}

			"r" | "run" | "flee" => {
				println!("You flee like the coward you are");
				if let Some(Event::Lose) = ctl.run_enemy_attack_during_flee() {
					println!("you lose?");
				}

				break;
			}

			_ => {
				println!("the fuck that mean?");
			}
		}
	}

	if let Some(enemy) = ctl.enemy {
		if enemy.health > 0 {
			ctx.hack_game_mut().update_enemy(loc, enemy);
		} else {
			ctx.hack_game_mut().remove_encounter_at(loc);
		}
	} else {
		ctx.hack_game_mut().remove_encounter_at(loc);	
	}

	println!("[battle] leave");
}





enum Event {
	Leave,
	Lose,
}
