use crate::prelude::*;
use crate::controller::*;
use crate::game_state::*;
use crate::enemy::*;
use crate::task;

#[derive(Debug)]
pub struct BattleController {
	location: Location,
	enemy: Enemy,
}


#[derive(Debug, Copy, Clone)]
enum AttackSeverity {
	Crit,
	Hit,
	Miss,
}

#[derive(Debug)]
pub enum PlayerCommand {
	Attack, Heal, Flee,
}


impl BattleController {
	pub fn new(location: Location, enemy: Enemy) -> BattleController {
		BattleController { location, enemy }
	}

	fn is_enemy_boss(&self) -> bool { self.enemy.archetype.is_boss() }

	async fn run_player_attack(&mut self) -> Option<Event> {
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

				self.apply_player_attack(severity).await
			}

			Ordering::Equal => {
				println!("Your weapons clash and neither you nor the {:?} take damage", self.enemy.archetype);
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

				self.apply_enemy_attack(severity, false).await
			}
		}

	}

	async fn run_enemy_attack_during_heal(&self) -> Option<Event> {
		use AttackSeverity::*;

		let probabilities = if self.is_enemy_boss() {[2, 2, 6]} else {[3, 4, 6]};
		let severity = choose_with_weights(&[Crit, Hit, Miss], &probabilities);

		self.apply_enemy_attack(severity, false).await
	}

	async fn run_enemy_attack_during_flee(&self) -> Option<Event> {
		if rng().gen_ratio(2, 5) {
			self.apply_enemy_attack(AttackSeverity::Hit, true).await
		} else {
			None
		}
	}


	async fn apply_player_attack(&mut self, severity: AttackSeverity) -> Option<Event> {
		let enemy_archetype = self.enemy.archetype;
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

		self.enemy.health -= damage.max(0);

		if self.enemy.is_dead() {
			println!("The strike is fatal! The {:?} is defeated!", enemy_archetype);
			Some(Event::Leave)
		} else {
			None
		}
	}


	async fn apply_enemy_attack(&self, severity: AttackSeverity, ignore_shield: bool) -> Option<Event> {
		let mut damage = self.enemy.archetype.attack();

		match severity {
			AttackSeverity::Crit => {
				println!("The {:?} strikes you and you take critical damage!", self.enemy.archetype);
				damage *= 2;
			}

			AttackSeverity::Hit => {
				println!("The {:?} strikes you!", self.enemy.archetype);
			}

			AttackSeverity::Miss => {
				println!("The {:?} swings at you but misses", self.enemy.archetype);
				return None
			}
		}

		let player_defense = get_coordinator().hack_game_mut().player.defense();
		let shield_applied = player_defense > 0 && rng().gen_ratio(3, 4);

		if shield_applied && !ignore_shield {
			println!("You raise your shield in time to take some of the blow");
			damage -= player_defense;
		}

		if !task::damage_player(damage.max(0) as u32, HealthModifyReason::Attack).await {
			println!("Unfortunately, the strike is fatal");
			Some(Event::Lose)
		} else {
			None
		}
	}
}


pub async fn run_battle_controller() {
	println!("[battle] enter");

	let loc = get_coordinator().hack_game().player.location;
	let enemy = get_coordinator().hack_game().get_enemy(loc)
		.expect("Tried to start battle with no enemy");

	let mut ctl = BattleController::new(loc, enemy);

	if ctl.enemy.archetype.is_boss() {
		println!("Oh fuck it's a boss!");
		println!("The {:?} sets its eyes on you", enemy.archetype);
	} else {
		println!("Oh shit it's a monster");
		println!("The {:?} readies its weapon", enemy.archetype);
	}

	println!("Do you fight or run like a coward?");

	while !ctl.enemy.is_dead() {
		let command = task::get_player_command().await;

		match command.battle().unwrap() {
			PlayerCommand::Attack => match ctl.run_player_attack().await {
				Some(Event::Lose) => {
					println!("lose??");
					break
				}

				Some(Event::Leave) => { break }

				_ => {}
			}

			PlayerCommand::Heal => {
				if task::consume_player_item(Item::Food).await {
					task::heal_player(rng().gen_range(1, 4)).await;

					if let Some(Event::Lose) = ctl.run_enemy_attack_during_heal().await {
						println!("Lose ???");
						break;
					}

				} else {
					println!("You don't have enough food!");
				}
			}

			PlayerCommand::Flee => {
				println!("You flee like the coward you are");
				if let Some(Event::Lose) = ctl.run_enemy_attack_during_flee().await {
					println!("you lose?");
				}

				break;
			}
		}
	}

	if ctl.enemy.health > 0 {
		get_coordinator().hack_game_mut().update_enemy(loc, ctl.enemy);
	} else {
		get_coordinator().hack_game_mut().remove_encounter_at(loc);
	}

	println!("[battle] leave");
}





enum Event {
	Leave,
	Lose,
}
