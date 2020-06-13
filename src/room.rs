use crate::prelude::*;
use rand::distributions::{Standard, Distribution};

#[derive(Debug, Copy, Clone)]
pub struct Room {
	pub doors: [bool; 4],
	pub encounter: Option<EncounterType>,
	pub is_exit: bool,
}

impl Room {
	pub fn new() -> Room {
		Room {
			doors: [false; 4],
			encounter: None,
			is_exit: false,
		}
	}

	pub fn door(&self, dir: Direction) -> bool { self.doors[dir as usize] }
	pub fn set_door(&mut self, dir: Direction, open: bool) { self.doors[dir as usize] = open; }

	pub fn iter_neighbor_directions(&self) -> impl Iterator<Item=Direction> + '_ {
		Direction::iter_all()
			.filter(move |&dir| self.door(dir))
	}

	pub fn has_interactable(&self) -> bool {
		self.is_exit || matches!(
			self.encounter,
			Some(EncounterType::Merchant) | Some(EncounterType::Chest)
		)
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

	/// Determines whether this encounter hangs around after the player
	/// enters the room or if it's consumed immediately
	pub fn is_persistent(&self) -> bool {
		match self {
			EncounterType::Merchant
			| EncounterType::Chest
			| EncounterType::Trap
			| EncounterType::Monster
			| EncounterType::Boss => true,

			_ => false
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

