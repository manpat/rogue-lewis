use crate::prelude::*;
use crate::map::Map;
use crate::room::Room;

#[derive(Debug)]
pub struct GameState {
	pub map: Map,
	pub player_location: Location,
	// inventory
	// stats/health
}


impl GameState {
	pub fn new() -> GameState {
		GameState {
			map: Map::new(),
			player_location: Location(0, 0),
		}
	}

	pub fn generate_room_at(&mut self, location: Location) -> &mut Room {
		if !self.map.has(location) {
			let mut room = Room::with_doors(random());

			for dir in room.iter_neighbor_directions() {
				let target_loc = location.offset_in_direction(dir);

				if let Some(target_room) = self.map.get_mut(target_loc) {
					target_room.set_door(dir.opposite(), true);
				}
			}

			for (dir, target_room) in self.map.iter_neighbors(location) {
				if target_room.door(dir.opposite()) {
					room.set_door(dir, true);
				}
			}

			self.map.add(location, room);
		}

		self.map.get_mut(location).unwrap()
	}

	pub fn try_move_player(&mut self, dir: Direction) -> bool {
		let room = self.map.get(self.player_location)
			.expect("Player somehow not in a room");

		if room.door(dir) {
			self.player_location = self.player_location.offset_in_direction(dir);
			self.generate_room_at(self.player_location);
			true
		} else {
			false
		}
	}
}