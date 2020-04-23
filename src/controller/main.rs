use crate::prelude::*;
use crate::controller::*;
use crate::game_state::{Item, HealthModifyReason};
use crate::room::EncounterType;
use crate::task;


async fn try_move(dir: Direction) -> bool {
	if !task::move_player(dir).await {
		println!("You can't go that way");
		return false;
	}

	if !task::consume_player_item(Item::Food).await {
		if !task::damage_player(1, HealthModifyReason::Hunger).await {
			println!("You starve to death");
			return true
		} else {
			println!("You have run out of food! You can travel {} rooms",
				get_coordinator().hack_game_mut().player.hunger);
		}
	} else {
		get_coordinator().hack_game_mut().player.hunger = 10;
	}

	let player_pos = get_coordinator().hack_game_mut().player.location;
	let current_room = get_coordinator().hack_game_mut().map.get(player_pos).unwrap();

	get_coordinator().hack_game_mut().map.mark_visited(player_pos);

	// TODO: leaving should be optional
	if current_room.is_exit {
		println!("You found the exit!");
		return true
	}

	if let Some(encounter_ty) = current_room.encounter {
		run_encounter(encounter_ty).await;

		if !encounter_ty.is_persistent() {
			get_coordinator().hack_game_mut().remove_encounter_at(player_pos);
		}
	}

	task::show_map(false).await;

	false
}

async fn run_encounter(encounter_ty: EncounterType) {
	println!("]]] running encounter {:?}", encounter_ty);

	match encounter_ty {
		EncounterType::Food => task::give_player_item(Item::Food).await,
		EncounterType::Treasure => task::give_player_item(Item::Treasure).await,
		EncounterType::Key => task::give_player_item(Item::Key).await,
		EncounterType::Map => task::give_player_item(Item::Map).await,

		// TODO: Chest should be optionally
		EncounterType::Chest => {
			let chest_items = [Item::Food, Item::Treasure, Item::Key];

			if task::consume_player_item(Item::Key).await {
				let num_items = rng().gen_range(1, 5);
				let items = chest_items.choose_multiple(&mut rng(), num_items);

				println!("You found a chest!");
				println!("You open it with one of your keys");

				for item in items {
					task::give_player_item(*item).await;
				}

			} else {
				println!("You found a chest, but don't have a key to open it");
			}
		}

		EncounterType::Monster => {
			let player_loc = get_coordinator().hack_game().player.location;
			if get_coordinator().hack_game_mut().get_enemy(player_loc).is_none() {
				get_coordinator().hack_game_mut().spawn_enemy_at(player_loc, false);
			}

			run_battle_controller().await;
		}

		EncounterType::Boss => {
			let player_loc = get_coordinator().hack_game().player.location;
			if get_coordinator().hack_game_mut().get_enemy(player_loc).is_none() {
				get_coordinator().hack_game_mut().spawn_enemy_at(player_loc, true);
			}

			run_battle_controller().await;
		}

		EncounterType::Merchant => {
			run_merchant_controller().await;
		}		

		_ => {}
	}
}

fn print_help() {
	println!("pls implement help");
}



pub async fn run_main_controller() {
	println!("[main] enter");

	// TODO: this doesn't make sense for a retained mode view
	task::show_map(false).await;

	'main_loop: while !get_coordinator().hack_game().player.is_dead() {
		// TODO: this should be moved to view, when input is requested
		println!("Which way do you go?");

		loop {
			let command = task::get_player_command().await;
			let command: Vec<&str> = command.0.split_whitespace().collect();

			if command[0] == "d" {
				let coordinator = get_coordinator().clone();
				let mut state = coordinator.hack_game_mut();
				use crate::room::Room;

				let ply_loc = state.player.location;
				let room = state.map.get(ply_loc).unwrap();

				match command[1..] {
					["state"] => println!("{:#?}", state),
					["ply"] => {
						println!("{:#?}", state.player);
						println!("attack {:#?}", state.player.attack());
						println!("defense {:#?}", state.player.defense());
					}
					["inv"] => println!("{:#?}", state.player.inventory),

					["room"] => println!("{:#?}", state.map.get(state.player.location)),
					["enemy"] => println!("{:#?}", state.get_enemy(state.player.location)),

					["g", "key"] => state.player.inventory.add(Item::Key),
					["g", "key", n] => state.player.inventory.add_n(Item::Key, n.parse().unwrap()),
					["g", "food"] => state.player.inventory.add(Item::Food),
					["g", "food", n] => state.player.inventory.add_n(Item::Food, n.parse().unwrap()),
					["g", "map"] => state.player.inventory.add(Item::Map),
					["g", "treasure"] => state.player.inventory.add(Item::Treasure),
					["g", "treasure", n] => state.player.inventory.add_n(Item::Treasure, n.parse().unwrap()),

					["g", "health", n] => { state.player.health += n.parse::<i32>().unwrap() }

					["p", "chest"] => {
						state.map.replace(ply_loc, Room {
							encounter: Some(EncounterType::Chest),
							.. room
						})
					}

					["p", "exit"] => {
						state.map.replace(ply_loc, Room {
							is_exit: true,
							.. room
						})
					}

					["battle"] => {
						drop(state);

						let loc = get_coordinator().hack_game_mut().player.location;
						get_coordinator().hack_game_mut().spawn_enemy_at(loc, random());

						run_battle_controller().await
					}

					["merchant"] => {
						drop(state);

						run_merchant_controller().await
					}

					_ => {
						println!("Nani!?");
					}
				}

				continue
			}

			match command[0] {
				"n" | "north" => if try_move(Direction::North).await { break 'main_loop }
				"e" | "east" => if try_move(Direction::East).await { break 'main_loop }
				"s" | "south" => if try_move(Direction::South).await { break 'main_loop }
				"w" | "west" => if try_move(Direction::West).await { break 'main_loop }
				"m" | "map" => task::show_map(true).await,

				"h" | "help" => print_help(),

				"heal" | "eat" => {
					if task::consume_player_item(Item::Food).await {
						task::heal_player(rng().gen_range(1, 4)).await;
					} else {
						println!("You don't have enough food!");
					}
				}

				// "r" | "restart" => Some(Event::Restart),
				"q" | "quit" => break 'main_loop,
				cmd => {
					println!("what now? '{}'", cmd);
					continue;
				}
			}

			break
		}
	}

	println!("[main] leave");
}