use crate::prelude::*;
use crate::controller::*;
use crate::game_state::{GameState, Item};
use crate::room::EncounterType;
use crate::task::Coordinator;


async fn try_move(ctx: Coordinator, dir: Direction) -> bool {
	if !ctx.hack_game_mut().try_move_player(dir) {
		println!("You can't go that way");
		return false
	}

	println!("You move {}", dir);

	if !ctx.hack_game_mut().player.inventory.take(Item::Food) {
		ctx.hack_game_mut().player.hunger -= 1;
		if ctx.hack_game_mut().player.hunger <= 0 {
			println!("You starve to death");
			return true
		} else {
			println!("You have run out of food! You can travel {} rooms", ctx.hack_game_mut().player.hunger);
		}
	} else {
		ctx.hack_game_mut().player.hunger = 10;
	}

	let player_pos = ctx.hack_game_mut().player.location;
	let current_room = ctx.hack_game_mut().map.get(player_pos).unwrap();

	ctx.hack_game_mut().map.mark_visited(player_pos);

	if current_room.is_exit {
		println!("You found the exit!");
		return true
	}

	if let Some(encounter_ty) = current_room.encounter {
		run_encounter(ctx.clone(), encounter_ty).await;

		if !encounter_ty.is_persistent() {
			ctx.hack_game_mut().remove_encounter_at(player_pos);
		}
	}

	print_local_area(&ctx.hack_game());

	false
}

async fn run_encounter(ctx: Coordinator, encounter_ty: EncounterType) {
	println!("]]] running encounter {:?}", encounter_ty);

	let player_loc = ctx.hack_game().player.location;

	match encounter_ty {
		EncounterType::Food => {
			ctx.hack_game_mut().player.inventory.add(Item::Food);
			println!("You found food");
		}

		EncounterType::Treasure => {
			ctx.hack_game_mut().player.inventory.add(Item::Treasure);
			println!("You found treasure");
		}

		EncounterType::Key => {
			ctx.hack_game_mut().player.inventory.add(Item::Key);
			println!("You found a key!");
		}

		EncounterType::Map => {
			if !ctx.hack_game_mut().player.inventory.has(Item::Map) {
				println!("You found a map!");
			} else {
				println!("You found another map. It may have some value");
			}

			ctx.hack_game_mut().player.inventory.add(Item::Map);
		}

		EncounterType::Chest => {
			let chest_items = [Item::Food, Item::Treasure, Item::Key];

			if ctx.hack_game_mut().player.inventory.take(Item::Key) {
				let num_items = rng().gen_range(1, 5);
				let items = chest_items.choose_multiple(&mut rng(), num_items);

				println!("You found a chest!");
				println!("You open it with one of your keys");

				for item in items {
					println!("You found a {:?}", item);
					ctx.hack_game_mut().player.inventory.add(*item);
				}

			} else {
				println!("You found a chest, but don't have a key to open it");
			}
		}

		EncounterType::Monster => {
			if ctx.hack_game_mut().get_enemy(player_loc).is_none() {
				ctx.hack_game_mut().spawn_enemy_at(player_loc, false);
			}

			run_battle_controller(ctx, player_loc).await;
		}

		EncounterType::Boss => {
			if ctx.hack_game_mut().get_enemy(player_loc).is_none() {
				ctx.hack_game_mut().spawn_enemy_at(player_loc, true);
			}

			run_battle_controller(ctx, player_loc).await;
		}

		EncounterType::Merchant => {
			run_merchant_controller(ctx).await;
		}		

		_ => {}
	}
}

fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", crate::rendering::render_map(&state, state.map.bounds()));
	println!("=============");
}

fn print_local_area(state: &GameState) {
	let bounds = state.map.iter()
		.filter(|(loc, _)| loc.distance(state.player.location) < 2)
		.fold(Bounds::empty(), |bounds, (loc, _)| bounds.include(loc))
		.expand(1, 0);

	println!("=============");
	println!("{}", crate::rendering::render_map(&state, bounds));
	println!("=============");
}

fn print_help() {
	println!("pls implement help");
}



pub async fn run_main_controller(ctx: Coordinator) {
	println!("[main] enter");

	'main_loop: loop {
		println!("Which way do you go?");
		print_local_area(&ctx.hack_game());

		loop {
			let command = ctx.get_player_command().await;
			let command: Vec<&str> = command.split_whitespace().collect();

			if command[0] == "d" {
				let mut state = ctx.hack_game_mut();
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
				"n" | "north" => if try_move(ctx.clone(), Direction::North).await { break 'main_loop }
				"e" | "east" => if try_move(ctx.clone(), Direction::East).await { break 'main_loop }
				"s" | "south" => if try_move(ctx.clone(), Direction::South).await { break 'main_loop }
				"w" | "west" => if try_move(ctx.clone(), Direction::West).await { break 'main_loop }
				"m" | "map" => print_map(&ctx.hack_game()),

				"h" | "help" => print_help(),

				// TODO: eat | heal

				"testbattle" => {
					let loc = ctx.hack_game_mut().player.location;
					ctx.hack_game_mut().spawn_enemy_at(loc, random());

					run_battle_controller(ctx.clone(), loc).await
				},
				"testmerchant" => run_merchant_controller(ctx.clone()).await,

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