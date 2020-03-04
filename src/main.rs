mod prelude;
mod types;
mod game_state;
mod map;
mod room;
mod rendering;

use prelude::*;

fn main() {
	let mut state = generate_game_state();

	loop {
		let bounds = state.map.iter()
			.filter(|(loc, _)| loc.distance(state.player_location) < 2)
			.fold(Bounds::empty(), |bounds, (loc, _)| bounds.include(loc))
			.expand(1, 0);

		println!("=============");
		println!("{}", rendering::render_map(&state, bounds));
		println!("=============");

		let mut try_move = |dir| {
			let room = state.map.get(state.player_location)
				.expect("Player somehow not in a room");

			if room.door(dir) {
				state.player_location = state.player_location.offset_in_direction(dir);
				state.generate_room_at(state.player_location);

				println!("You move {}", dir);
			} else {
				println!("You can't go that way");
			}
		};

		let mut command = read_line();
		command.make_ascii_lowercase();

		match command.as_str() {
			"n" => try_move(Direction::North),
			"e" => try_move(Direction::East),
			"s" => try_move(Direction::South),
			"w" => try_move(Direction::West),
			"r" => { state = generate_game_state(); }
			"m" => {
				println!("==== map ====");
				println!("{}", rendering::render_map(&state, state.map.bounds()));
				println!("=============");
			}

			"quit" | "q" => break,
			_ => println!("what now?")
		}
	}
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