use crate::prelude::*;
use crate::room::Room;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Map {
	rooms: HashMap<Location, Room>,
}

impl Map {
	pub fn new() -> Map {
		Map {
			rooms: [(Location(0, 0), Room::new())].iter().cloned().collect()
		}
	}

	pub fn add(&mut self, loc: Location, room: Room) {
		let prev = self.rooms.insert(loc, room);
		assert!(prev.is_none(), "Room already exists");
	}

	pub fn has(&self, loc: Location) -> bool { self.rooms.contains_key(&loc) }
	pub fn get(&self, loc: Location) -> Option<&Room> { self.rooms.get(&loc) }
	pub fn get_mut(&mut self, loc: Location) -> Option<&mut Room> { self.rooms.get_mut(&loc) }

	pub fn bounds(&self) -> Bounds {
		self.rooms.keys()
			.fold(Bounds::empty(), |bounds, loc| bounds.include(*loc))
	}

	pub fn iter(&self) -> impl Iterator<Item=(Location, &Room)> {
		self.rooms.iter()
			.map(|(loc, room)| (*loc, room))
	}

	pub fn iter_neighbors(&self, location: Location) -> impl Iterator<Item=(Direction, &Room)> {
		Direction::iter_all()
			.filter_map(move |dir| {
				let room_loc = location.offset_in_direction(dir);
				let room = self.rooms.get(&room_loc)?;
				Some((dir, room))
			})
	}

}