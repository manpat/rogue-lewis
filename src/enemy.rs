use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
pub enum EnemyArchetype {
	Goblin,
	Ogre,
	Orc,
	Gargoyle,
	Guardian,
	Minotaur,
}

#[derive(Debug, Copy, Clone)]
pub struct Enemy {
	pub archetype: EnemyArchetype,
	pub health: i32,
}


use EnemyArchetype::*;

impl EnemyArchetype {
	pub fn choose(boss: bool) -> Self {
		let options: &[EnemyArchetype] = if boss {
			&[Guardian, Minotaur]
		} else {
			&[Goblin, Ogre, Orc, Gargoyle]
		};

		*options.choose(&mut rng()).unwrap()
	}

	pub fn health(self) -> i32 {
		match self {
			Goblin => 3,
			Ogre => 6,
			Orc => 5,
			Gargoyle => 2,
			
			Guardian => 15,
			Minotaur => 9,
		}
	}

	pub fn attack(self) -> i32 {
		match self {
			Goblin => 1,
			Ogre => 1,
			Orc => 2,
			Gargoyle => 4,
			
			Guardian => 1,
			Minotaur => 4,
		}
	}

	pub fn defense(self) -> i32 {
		match self {
			Guardian => 2,
			Minotaur => 1,
			_ => 0
		}
	}

	pub fn is_boss(self) -> bool {
		matches!(self, Guardian | Minotaur)
	}

	pub fn new(self) -> Enemy {
		Enemy { archetype: self, health: self.health() }
	}
}