#![feature(nll)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![deny(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]

mod prelude;
mod types;
mod game_state;
mod map;
mod room;
mod rendering;
mod controller;
mod enemy;

use prelude::*;
use controller::{MainController, Controller, ControllerContext, Event};

fn main() {
	let mut state = generate_game_state();
	let mut main_controller = Controller::from(MainController);
	main_controller.enter(&mut ControllerContext::new(&mut state));

	let mut controllers: Vec<Controller> = vec![main_controller];

	loop {
		let mut command = read_command();
		command.make_ascii_lowercase();
		let command = command.split_whitespace().collect::<Vec<_>>();

		if command.is_empty() {
			continue;
		}

		if command[0] == "d" {
			use game_state::Item;
			use room::{Room, EncounterType};

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
				["ctl"] => println!("{:#?}", controllers),

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

		let ctl_ctx = &mut ControllerContext::new(&mut state);

		match controllers.last_mut().unwrap().run_command(ctl_ctx, command[0]) {
			None => {}

			Some(Event::TransitionTo(mut new)) => {
				if let Some(mut prev) = controllers.pop() {
					prev.leave(ctl_ctx);
				}
				new.enter(ctl_ctx);
				controllers.push(new);
			}

			Some(Event::Enter(mut new)) => {
				controllers.last_mut().unwrap().leave(ctl_ctx);

				new.enter(ctl_ctx);
				controllers.push(new);
			}

			Some(Event::Leave) => {
				if let Some(mut prev) = controllers.pop() {
					prev.leave(ctl_ctx);
				}

				controllers.last_mut().unwrap().enter(ctl_ctx);
			}

			Some(Event::Restart) => {
				println!("The walls warp and shift around you and your sense of reality temporarily disolves");
				println!("You are unsure if any of the events you've experienced until now actually happened");
				*ctl_ctx.state = generate_game_state();
				let mut main_controller = Controller::from(MainController);
				main_controller.enter(ctl_ctx);
				controllers = vec![main_controller];
			}

			Some(Event::Win) => {
				println!("You win!");
				break
			}
			Some(Event::Lose) => {
				println!("You lost");
				break
			}
			Some(Event::Quit) => {
				println!("See ya!");
				break
			}
		}
	}
}


fn read_command() -> String {
	use std::io::{Write, BufRead};

	print!("> ");

	std::io::stdout().flush()
		.expect("Failed to flush");

	std::io::stdin().lock()
		.lines().next()
		.expect("EOF")
		.expect("Failed to read stdin")
}


fn generate_game_state() -> game_state::GameState {
	let mut state = game_state::GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}