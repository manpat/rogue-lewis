use crate::prelude::*;
use crate::gamestate::*;
use crate::enemy::*;
use crate::item::*;
use crate::task;


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


async fn run_player_attack(enemy_archetype: EnemyArchetype) {
	use std::cmp::Ordering;
	use AttackSeverity::*;

	let mut r = rng();

	let enemy_max_roll = if enemy_archetype.is_boss() { 13 } else { 10 };

	let player_roll = r.gen_range(0, 10);
	let enemy_roll = r.gen_range(0, enemy_max_roll);

	match player_roll.cmp(&enemy_roll) {
		Ordering::Greater => {
			let severity = match player_roll {
				1 => Miss,
				9..=10 => Crit,
				_ => Hit,
			};

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
					return
				}
			}

			let enemy_defense = enemy_archetype.defense();
			let shield_applied = enemy_defense > 0 && rng().gen_ratio(3, 4);

			if shield_applied {
				println!("The {:?} raises it's shield and blocks some of your attack", enemy_archetype);
				damage -= enemy_defense;
			}

			task::attack_enemy(damage.max(0)).await;
		}

		Ordering::Equal => {
			println!("Your weapons clash and neither you nor the {:?} take damage", enemy_archetype);
		}

		Ordering::Less => {
			let severity = if enemy_archetype.is_boss() {
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

			run_enemy_attack(enemy_archetype, severity, false).await
		}
	}
}


async fn run_enemy_attack(archetype: EnemyArchetype, severity: AttackSeverity, ignore_shield: bool) {
	let mut damage = archetype.attack();

	match severity {
		AttackSeverity::Crit => {
			println!("The {:?} strikes you and you take critical damage!", archetype);
			damage *= 2;
		}

		AttackSeverity::Hit => {
			println!("The {:?} strikes you!", archetype);
		}

		AttackSeverity::Miss => {
			println!("The {:?} swings at you but misses", archetype);
			return
		}
	}

	let player_defense = get_coordinator().hack_game_mut().player.defense();
	let shield_applied = player_defense > 0 && rng().gen_ratio(3, 4);

	if shield_applied && !ignore_shield {
		println!("You raise your shield in time to take some of the blow");
		damage -= player_defense;
	}

	task::damage_player(damage.max(0) as u32, HealthModifyReason::Attack).await;
}


pub async fn run_battle_controller() {
	println!("[battle] enter");

	let loc = get_coordinator().hack_game().player.location;
	let archetype = get_coordinator().hack_game().get_enemy(loc)
		.expect("Tried to start battle with no enemy")
		.archetype;

	if archetype.is_boss() {
		println!("Oh fuck it's a boss!");
		println!("The {:?} sets its eyes on you", archetype);
	} else {
		println!("Oh shit it's a monster");
		println!("The {:?} readies its weapon", archetype);
	}

	println!("Do you fight or run like a coward?");

	while !get_coordinator().hack_game().get_enemy(loc).unwrap().is_dead() && !get_coordinator().hack_game().player.is_dead() {
		let command = task::get_player_command().await;

		match command.battle().unwrap() {
			PlayerCommand::Attack => run_player_attack(archetype).await,

			PlayerCommand::Heal => {
				if task::consume_player_item(Item::Food).await {
					task::heal_player(rng().gen_range(1, 4)).await;

					use AttackSeverity::*;

					let probabilities = if archetype.is_boss() {[2, 2, 6]} else {[3, 4, 6]};
					let severity = choose_with_weights(&[Crit, Hit, Miss], &probabilities);

					run_enemy_attack(archetype, severity, false).await;

				} else {
					println!("You don't have enough food!");
				}
			}

			PlayerCommand::Flee => {
				println!("You flee like the coward you are");
				
				if rng().gen_ratio(2, 5) {
					run_enemy_attack(archetype, AttackSeverity::Hit, true).await;
				}

				break;
			}
		}
	}

	if get_coordinator().hack_game().get_enemy(loc).unwrap().is_dead() {
		println!("The strike is fatal! The {:?} is defeated!", archetype);
		get_coordinator().hack_game_mut().remove_encounter_at(loc);
	}

	println!("[battle] leave");
}
