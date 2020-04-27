use crate::prelude::*;
use rand::distributions::{Standard, Distribution};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
	Food,
	Treasure,
	Map,
	Key,
	Potion,

	Equipment(Equipment),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Equipment {
	Sword,
	Shield,
	Axe,
	Armour,
}


impl Equipment {
	pub fn attack_bonus(self) -> i32 {
		match self {
			Equipment::Sword => 1,
			Equipment::Axe => 2,
			_ => 0
		}
	}

	pub fn defense_bonus(self) -> i32 {
		match self {
			Equipment::Shield => 1,
			Equipment::Armour => 2,
			_ => 0
		}
	}
}


impl Distribution<Equipment> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Equipment {
		[
			Equipment::Sword,
			Equipment::Shield,
			Equipment::Axe,
			Equipment::Armour,
		].choose(rng).cloned().unwrap()
	}
}