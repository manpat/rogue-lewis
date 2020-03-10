#![feature(box_syntax)]
#![feature(vec_remove_item)]

mod prelude;
mod types;
mod game_state;
mod map;
mod room;
mod rendering;
mod controller;

use prelude::*;
use controller::{Controller, Event};

fn main() {
	let mut state = generate_game_state();
	let mut controller = box controller::MainController as Box<dyn Controller>;
	controller.init(&state);

	loop {
		let mut command = read_command();
		command.make_ascii_lowercase();

		if command.starts_with("dbg") {
			match command.trim_start_matches("dbg").trim() {
				"state" => println!("{:#?}", state),
				"ply" => println!("{:#?}", state.player),
				"inv" => println!("{:#?}", state.player.inventory),
				"ctl" => println!("{:#?}", controller),
				"key" => {
					state.player.inventory.add(game_state::Item::Key)
				}
				_ => {}
			}

			continue
		}

		match controller.run_command(&mut state, &command) {
			None => {}

			Some(Event::Transition(new)) => {
				controller = new;
				controller.init(&state);
			}

			Some(Event::Restart) => {
				println!("The walls warp and shift around you and your sense of reality temporarily disolves");
				println!("You are unsure if any of the events you've experienced until now actually happened");
				state = generate_game_state();
				controller = box controller::MainController;
				controller.init(&state);
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

	let mut builder_loc = Location(0, 0);

	for _ in 0..50 {
		// Walk through a door if possible, otherwise just pick a direction and pretend there's a door there
		let walk_dir = if let Some(room) = state.map.get(builder_loc) {
			room.iter_neighbor_directions().choose(&mut rng())
				.unwrap_or_else(|| random())
		} else {
			random()
		};

		builder_loc = builder_loc.offset_in_direction(walk_dir);
		let new_room = state.generate_room_at(builder_loc);
		new_room.set_door(walk_dir.opposite(), true);
	}

	// Fixup
	let old_map = state.map.clone();
	for (location, room) in old_map.iter() {
		for dir in room.iter_neighbor_directions() {
			if let Some(neighbor) = state.map.get_mut(location.offset_in_direction(dir)) {
				neighbor.set_door(dir.opposite(), true);
			}
		}
	}

	state
}