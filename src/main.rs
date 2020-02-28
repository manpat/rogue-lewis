mod prelude;
mod types;
mod game_state;
mod map;
mod room;
mod rendering;

use prelude::*;

fn main() {
	let mut state = game_state::GameState::new();

	{
		let mut builder_loc = Location(0, 0);

		for _ in 0..100 {
			// Walk through a door if possible, otherwise just pick a direction and pretend there's a door there
			let walk_dir = if let Some(room) = state.map.get(builder_loc) {
				room.neighbor_directions().choose(&mut rng())
					.unwrap_or_else(|| random())
			} else {
				random()
			};

			builder_loc = builder_loc.offset_in_direction(walk_dir);

			if let Some(room) = state.map.get_mut(builder_loc) {
				room.set_door(walk_dir.opposite(), true);
			} else {
				let mut room = room::Room::with_doors(rng().gen());
				room.set_door(walk_dir.opposite(), true);
				state.map.add(builder_loc, room);
			}
		}


		let old_map = state.map.clone();
		for (location, room) in old_map.iter() {
			for dir in room.neighbor_directions() {
				if let Some(neighbor) = state.map.get_mut(location.offset_in_direction(dir)) {
					neighbor.set_door(dir.opposite(), true);
				}
			}
		}
	}


	// println!("{:#?}", state);

	println!("==== map ====");
	println!("{}", rendering::render(&state));
	println!("=============");
}
