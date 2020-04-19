use crate::prelude::*;
use crate::controller::*;
use crate::game_state::{GameState, Item};
use crate::room::EncounterType;
use crate::task::{self, Coordinator};


async fn try_move(dir: Direction) -> bool {
	if !get_coordinator().hack_game_mut().try_move_player(dir) {
		println!("You can't go that way");
		return false
	}

	println!("You move {}", dir);

	if !get_coordinator().hack_game_mut().player.inventory.take(Item::Food) {
		get_coordinator().hack_game_mut().player.hunger -= 1;
		if get_coordinator().hack_game_mut().player.hunger <= 0 {
			println!("You starve to death");
			return true
		} else {
			println!("You have run out of food! You can travel {} rooms", get_coordinator().hack_game_mut().player.hunger);
		}
	} else {
		get_coordinator().hack_game_mut().player.hunger = 10;
	}

	let player_pos = get_coordinator().hack_game_mut().player.location;
	let current_room = get_coordinator().hack_game_mut().map.get(player_pos).unwrap();

	get_coordinator().hack_game_mut().map.mark_visited(player_pos);

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

	let player_loc = get_coordinator().hack_game().player.location;

	match encounter_ty {
		EncounterType::Food => {
			get_coordinator().hack_game_mut().player.inventory.add(Item::Food);
			println!("You found food");
		}

		EncounterType::Treasure => {
			get_coordinator().hack_game_mut().player.inventory.add(Item::Treasure);
			println!("You found treasure");
		}

		EncounterType::Key => {
			get_coordinator().hack_game_mut().player.inventory.add(Item::Key);
			println!("You found a key!");
		}

		EncounterType::Map => {
			if !get_coordinator().hack_game_mut().player.inventory.has(Item::Map) {
				println!("You found a map!");
			} else {
				println!("You found another map. It may have some value");
			}

			get_coordinator().hack_game_mut().player.inventory.add(Item::Map);
		}

		EncounterType::Chest => {
			let chest_items = [Item::Food, Item::Treasure, Item::Key];

			if get_coordinator().hack_game_mut().player.inventory.take(Item::Key) {
				let num_items = rng().gen_range(1, 5);
				let items = chest_items.choose_multiple(&mut rng(), num_items);

				println!("You found a chest!");
				println!("You open it with one of your keys");

				for item in items {
					println!("You found a {:?}", item);
					get_coordinator().hack_game_mut().player.inventory.add(*item);
				}

			} else {
				println!("You found a chest, but don't have a key to open it");
			}
		}

		EncounterType::Monster => {
			if get_coordinator().hack_game_mut().get_enemy(player_loc).is_none() {
				get_coordinator().hack_game_mut().spawn_enemy_at(player_loc, false);
			}

			run_battle_controller(player_loc).await;
		}

		EncounterType::Boss => {
			if get_coordinator().hack_game_mut().get_enemy(player_loc).is_none() {
				get_coordinator().hack_game_mut().spawn_enemy_at(player_loc, true);
			}

			run_battle_controller(player_loc).await;
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

	'main_loop: loop {
		println!("Which way do you go?");

		// TODO: this doesn't make sense for a retained mode view
		task::show_map(false).await;

		loop {
			let command = task::get_player_command().await;
			let command: Vec<&str> = command.split_whitespace().collect();

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

				// TODO: eat | heal

				"testbattle" => {
					let loc = get_coordinator().hack_game_mut().player.location;
					get_coordinator().hack_game_mut().spawn_enemy_at(loc, random());

					run_battle_controller(loc).await
				},
				"testmerchant" => run_merchant_controller().await,

				// "iwin" => Some(Event::Win),
				// "ilose" => Some(Event::Lose),

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