use crate::prelude::*;
use crate::controller::*;
use crate::item::*;
use crate::room::EncounterType;
use crate::task;


#[derive(Debug)]
pub enum PlayerCommand {
	GoNorth,
	GoEast,
	GoSouth,
	GoWest,

	Heal,

	ShowMap,
	Quit,
}



async fn try_move(dir: Direction) -> bool {
	if !task::move_player(dir).await {
		println!("You can't go that way");
		return false;
	}

	if !task::consume_player_item(Item::Food).await {
		if !task::starve_player().await {
			println!("You starve to death");
			return true
		} else {
			println!("You have run out of food! You can travel {} rooms",
				get_coordinator().hack_game_mut().player.hunger);
		}
	} else {
		task::sate_player().await;
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

		EncounterType::Equipment => task::give_player_item(Item::Equipment(random())).await,

		// TODO: Chest should be optionally
		EncounterType::Chest => {
			let chest_items = [
				Item::Food, Item::Treasure, Item::Key,
				Item::Potion, Item::Equipment(random())
			];

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

			task::enter_mode(task::ControllerMode::Battle).await;
			run_battle_controller().await;
			task::leave_mode().await;
		}

		EncounterType::Boss => {
			let player_loc = get_coordinator().hack_game().player.location;
			if get_coordinator().hack_game_mut().get_enemy(player_loc).is_none() {
				get_coordinator().hack_game_mut().spawn_enemy_at(player_loc, true);
			}

			task::enter_mode(task::ControllerMode::Battle).await;
			run_battle_controller().await;
			task::leave_mode().await;
		}

		EncounterType::Merchant => {
			task::enter_mode(task::ControllerMode::Merchant).await;
			run_merchant_controller().await;
			task::leave_mode().await;
		}		

		_ => {}
	}
}


pub async fn run_main_controller() {
	println!("[main] enter");

	task::enter_mode(task::ControllerMode::Main).await;

	// TODO: this doesn't make sense for a retained mode view
	task::show_map(false).await;

	'main_loop: while !get_coordinator().hack_game().player.is_dead() {
		// TODO: this should be moved to view, when input is requested
		println!("Which way do you go?");

		loop {
			let command = task::get_player_command().await;

			if let Some(command) = command.debug() {
				let coordinator = get_coordinator().clone();
				let mut state = coordinator.hack_game_mut();
				use crate::room::Room;

				let ply_loc = state.player.location;
				let room = state.map.get(ply_loc).unwrap();

				let command: Vec<&str> = command.iter().map(String::as_ref).collect();

				match &command[..] {
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

						task::enter_mode(task::ControllerMode::Battle).await;
						run_battle_controller().await;
						task::leave_mode().await;
					}

					["merchant"] => {
						drop(state);

						task::enter_mode(task::ControllerMode::Merchant).await;
						run_merchant_controller().await;
						task::leave_mode().await;
					}

					_ => {
						println!("Nani!?");
					}
				}

				continue
			}

			match command.main().unwrap() {
				PlayerCommand::GoNorth => if try_move(Direction::North).await { break 'main_loop }
				PlayerCommand::GoEast => if try_move(Direction::East).await { break 'main_loop }
				PlayerCommand::GoSouth => if try_move(Direction::South).await { break 'main_loop }
				PlayerCommand::GoWest => if try_move(Direction::West).await { break 'main_loop }
				PlayerCommand::ShowMap => task::show_map(true).await,

				PlayerCommand::Heal => {
					if task::consume_player_item(Item::Food).await {
						task::heal_player(rng().gen_range(1, 4)).await;
					} else {
						println!("You don't have enough food!");
					}
				}

				PlayerCommand::Quit => break 'main_loop,
			}

			break
		}
	}

	task::leave_mode().await;

	println!("[main] leave");
}