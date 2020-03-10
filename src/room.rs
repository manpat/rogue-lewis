use crate::prelude::*;
use rand::distributions::{Standard, Distribution};

#[derive(Debug, Clone)]
pub struct Room {
	pub doors: [bool; 4],
	pub encounter: Option<EncounterType>,
}

impl Room {
	pub fn with_doors(doors: [bool; 4]) -> Room { Room { doors, encounter: None } }
	pub fn new() -> Room { Room::with_doors([false; 4]) }

	pub fn door(&self, dir: Direction) -> bool { self.doors[dir as usize] }
	pub fn set_door(&mut self, dir: Direction, open: bool) { self.doors[dir as usize] = open; }

	pub fn iter_neighbor_directions(&self) -> impl Iterator<Item=Direction> + '_ {
		self.doors.iter().cloned()
			.enumerate()
			.filter(|&(_, door)| door)
			.map(|(idx, _)| idx.into())
	}
}


#[derive(Debug, Clone, Copy)]
pub enum EncounterType {
	Food,
	Treasure,
	Key,
	Map,
	Equipment,

	Merchant,
	Chest,

	Trap,
	Monster,
	Boss,
}

impl EncounterType {
	pub fn probability(&self) -> f32 {
		match self {
			EncounterType::Food 	=> 33.0,
			EncounterType::Treasure => 10.0,
			EncounterType::Key 		=> 2.6,
			EncounterType::Map 		=> 2.0,
			EncounterType::Equipment=> 4.0,
			EncounterType::Merchant => 3.3,
			EncounterType::Chest 	=> 3.0,
			EncounterType::Trap 	=> 5.0,
			EncounterType::Monster 	=> 13.0,
			EncounterType::Boss 	=> 1.0,
		}
	}
}

impl Distribution<EncounterType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EncounterType {
		let choices = [
			EncounterType::Food,
			EncounterType::Treasure,
			EncounterType::Key,
			EncounterType::Map,
			EncounterType::Merchant,
			EncounterType::Equipment,
			EncounterType::Chest,
			EncounterType::Trap,
			EncounterType::Monster,
			EncounterType::Boss,
		];

		*choices.choose_weighted(rng, EncounterType::probability).unwrap()
	}
}