use crate::prelude::*;
use crate::room::Room;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Map {
	rooms: HashMap<Location, Room>,
	visited: HashSet<Location>,
}

impl Map {
	pub fn new() -> Map {
		Map {
			rooms: [(Location(0, 0), Room::new())].iter().cloned().collect(),
			visited: [Location(0, 0)].iter().cloned().collect(),
		}
	}

	pub fn add(&mut self, loc: Location, room: Room) {
		let prev = self.rooms.insert(loc, room);
		assert!(prev.is_none(), "Room already exists");
	}

	pub fn replace(&mut self, loc: Location, room: Room) {
		let dst_room = self.rooms.get_mut(&loc).expect("Trying to replace room that doesn't exist");
		*dst_room = room;
	}

	pub fn has(&self, loc: Location) -> bool { self.rooms.contains_key(&loc) }
	pub fn get(&self, loc: Location) -> Option<Room> { self.rooms.get(&loc).cloned() }

	pub fn visited(&self, loc: Location) -> bool { self.visited.contains(&loc) }
	pub fn mark_visited(&mut self, loc: Location) -> bool { self.visited.insert(loc) }

	pub fn bounds(&self) -> Bounds {
		self.rooms.keys()
			.fold(Bounds::empty(), |bounds, loc| bounds.include(*loc))
	}

	pub fn iter(&self) -> impl Iterator<Item=(Location, Room)> + '_ {
		self.rooms.iter()
			.map(|(loc, room)| (*loc, room.clone()))
	}

	pub fn iter_neighbors(&self, location: Location) -> impl Iterator<Item=(Direction, Room)> + '_ {
		Direction::iter_all()
			.filter_map(move |dir| {
				let room_loc = location.offset_in_direction(dir);
				let room = self.rooms.get(&room_loc)?;
				Some((dir, room.clone()))
			})
	}
}


#[derive(Debug)]
pub struct MapBuilder<'m> {
	map: &'m mut Map
}

impl<'m> MapBuilder<'m> {
	pub fn new(map: &'m mut Map) -> MapBuilder<'m> {
		MapBuilder { map }
	}


	pub fn ensure_room_connected(&mut self, location: Location) {
		let mut center_room = match self.map.get(location) {
			Some(room) => room,
			None => return,
		};

		// Correct neighbors if this room has disconnected outgoing corridors
		for dir in center_room.iter_neighbor_directions() {
			let neighbor_loc = location.offset_in_direction(dir);

			if let Some(mut neighbor) = self.map.get(neighbor_loc) {
				neighbor.set_door(dir.opposite(), true);
				self.map.replace(neighbor_loc, neighbor);
			}
		}

		// Correct doors w/ disconnected incomming corridors
		for (dir, room) in self.map.iter_neighbors(location) {
			if room.door(dir.opposite()) {
				center_room.set_door(dir, true);
			}
		}

		self.map.replace(location, center_room);
	}

	pub fn generate_room_at(&mut self, location: Location) {
		if !self.map.has(location) {
			self.map.add(location, MapBuilder::generate_room());
			self.ensure_room_connected(location);
		}
	}

	fn generate_room() -> Room {
		Room {
			doors: random(),
			encounter: if rng().gen_bool(0.2) {
				Some(random())
			} else {
				None
			},
			is_exit: false,
		}
	}

	pub fn generate_random_walk(&mut self) {
		let mut builder_loc = Location(0, 0);

		for _ in 0..50 {
			// Walk through a door if possible, otherwise just pick a direction and pretend there's a door there
			let walk_dir = if let Some(room) = self.map.get(builder_loc) {
				room.iter_neighbor_directions().choose(&mut rng())
					.unwrap_or_else(|| random())
			} else {
				random()
			};

			builder_loc = builder_loc.offset_in_direction(walk_dir);

			if !self.map.has(builder_loc) {
				let mut room = MapBuilder::generate_room();
				room.set_door(walk_dir.opposite(), true);
				self.map.add(builder_loc, room);

				self.ensure_room_connected(builder_loc);
			}
		}

		// Find a place to spawn the exit and spawn it
		loop {
			let (loc, mut room) = self.map.iter().choose(&mut rng()).unwrap();
			if loc != Location(0, 0) {
				room.encounter = None; // TODO: boss
				room.is_exit = true;

				self.map.replace(loc, room);
				break;
			}
		}
	}
}