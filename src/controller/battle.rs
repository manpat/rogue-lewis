use crate::prelude::*;
use crate::controller::*;
use crate::game_state::*;
use crate::enemy::*;

#[derive(Debug)]
pub struct BattleController {
	location: Location,
	enemy: Option<Enemy>,
}

#[derive(Copy, Clone, Debug)]
pub enum BattleEvent {

}

impl ControllerTrait for BattleController {
	fn enter(&mut self, ctx: &mut ControllerContext<'_>) {
		let enemy = ctx.state.get_enemy(self.location)
			.expect("Tried to start battle with no enemy");

		self.enemy = Some(enemy);

		if enemy.archetype.is_boss() {
			println!("Oh fuck it's a boss!");
			println!("The {:?} sets its eyes on you", enemy.archetype);
		} else {
			println!("Oh shit it's a monster");
			println!("The {:?} readies its weapon", enemy.archetype);
		}

		println!("Do you fight or run like a coward?");
	}

	fn leave(&mut self, ctx: &mut ControllerContext<'_>) {
		if let Some(enemy) = self.enemy {
			if enemy.health > 0 {
				ctx.state.update_enemy(self.location, enemy);
			} else {
				ctx.state.remove_encounter_at(self.location);
			}
		} else {
			ctx.state.remove_encounter_at(self.location);	
		}
	}

	fn run_command(&mut self, ctx: &mut ControllerContext<'_>, command: &str) -> Option<Event> {
		match command {
			"f" | "fight" => self.run_player_attack(ctx),

			"e" | "eat" | "h" | "heal" => {
				if ctx.state.player.inventory.take(Item::Food) {
					let health_gain: i32 = rng().gen_range(1, 4);
					ctx.state.player.health += health_gain;
					println!("You recover {} health", health_gain);

					self.run_enemy_attack_during_heal(ctx)

				} else {
					println!("You don't have enough food!");
					None
				}
			}

			"r" | "run" | "flee" => {
				println!("You flee like the coward you are");
				self.run_enemy_attack_during_flee(ctx)
					.or(Some(Event::Leave))
			}

			_ => {
				println!("the fuck that mean");
				None
			}
		}
	}
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

	fn run_player_attack(&mut self, ctx: &mut ControllerContext<'_>) -> Option<Event> {
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

				self.apply_player_attack(ctx, severity)
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

				self.apply_enemy_attack(ctx, severity, false)
			}
		}

	}

	fn run_enemy_attack_during_heal(&self, ctx: &mut ControllerContext<'_>) -> Option<Event> {
		use AttackSeverity::*;

		let probabilities = if self.is_enemy_boss() {[2, 2, 6]} else {[3, 4, 6]};
		let severity = choose_with_weights(&[Crit, Hit, Miss], &probabilities);

		self.apply_enemy_attack(ctx, severity, false)
	}

	fn run_enemy_attack_during_flee(&self, ctx: &mut ControllerContext<'_>) -> Option<Event> {
		if rng().gen_ratio(2, 5) {
			self.apply_enemy_attack(ctx, AttackSeverity::Hit, true)
		} else {
			None
		}
	}


	fn apply_player_attack(&mut self, ctx: &mut ControllerContext<'_>, severity: AttackSeverity) -> Option<Event> {
		let enemy_archetype = self.enemy_archetype();
		let player = &ctx.state.player;
		let mut damage = player.attack();

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


	fn apply_enemy_attack(&self, ctx: &mut ControllerContext<'_>, severity: AttackSeverity, ignore_shield: bool) -> Option<Event> {
		let enemy_archetype = self.enemy_archetype();
		let player = &mut ctx.state.player;
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