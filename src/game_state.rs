use crate::prelude::*;
use crate::map::{Map, MapBuilder};
use crate::room::Room;

#[derive(Debug)]
pub struct GameState {
	pub map: Map,
	pub player: Player,
}


impl GameState {
	pub fn new() -> GameState {
		GameState {
			map: Map::new(),
			player: Player::new(),
		}
	}

	pub fn try_move_player(&mut self, dir: Direction) -> bool {
		let room = self.map.get(self.player.location)
			.expect("Player somehow not in a room");

		if room.door(dir) {
			self.player.location = self.player.location.offset_in_direction(dir);
			MapBuilder::new(&mut self.map).generate_room_at(self.player.location);
			true
		} else {
			false
		}
	}

	pub fn remove_encounter_at(&mut self, loc: Location) {
		if let Some(room) = self.map.get(loc) {
			self.map.replace(loc, Room { encounter: None, .. room });
		}
	}
}



#[derive(Debug)]
pub struct Player {
	pub location: Location,
	pub health: i32,
	pub hunger: i32,

	pub inventory: Inventory,
}

impl Player {
	pub fn new() -> Self {
		let mut inventory = Inventory::new();
		inventory.add_n(Item::Food, 20);
		inventory.add_n(Item::Treasure, 5);

		Player {
			location: Location(0, 0),
			health: 15,
			hunger: 10,

			inventory,
		}
	}

	pub fn attack(&self) -> i32 { 2 }
	pub fn defense(&self) -> i32 { 0 }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Item {
	Food,
	Treasure,
	Map,
	Key,
}


#[derive(Debug)]
pub struct Inventory {
	items: Vec<Item>,
	treasure: i32,
	food: i32,
}

impl Inventory {
	pub fn new() -> Inventory {
		Inventory {
			items: Vec::new(),
			treasure: 0,
			food: 0,
		}
	}

	pub fn add(&mut self, item: Item) { self.add_n(item, 1) }

	pub fn add_n(&mut self, item: Item, n: usize) {
		match item {
			Item::Treasure => { self.treasure += n as i32 }
			Item::Food => { self.food += n as i32 }
			_ => {
				use std::iter;
				self.items.extend(iter::repeat(item).take(n))
			}
		}
	}

	pub fn take(&mut self, item: Item) -> bool { self.take_n(item, 1) }

	pub fn take_n(&mut self, item: Item, n: usize) -> bool {
		if self.count(item) < n {
			return false
		}

		match item {
			Item::Treasure => self.treasure -= n as i32,
			Item::Food => self.food -= n as i32,
			_ => for _ in 0..n {
				self.items.remove_item(&item);
			}
		}

		true
	}

	pub fn count(&self, item: Item) -> usize {
		match item {
			Item::Treasure => self.treasure as usize,
			Item::Food => self.food as usize,
			_ => self.items.iter()
				.filter(|&&i| i == item)
				.count()
		}
	}

	pub fn has(&self, item: Item) -> bool { self.count(item) > 0 }
}