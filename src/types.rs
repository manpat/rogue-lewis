use crate::prelude::*;
use rand::distributions::{Standard, Distribution};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Location(pub i32, pub i32);

impl Location {
	pub fn offset(self, dx: i32, dy: i32) -> Location {
		Location(self.0 + dx, self.1 + dy)
	}

	pub fn scale(self, dx: i32, dy: i32) -> Location {
		Location(self.0 * dx, self.1 * dy)
	}

	pub fn relative_to(self, Location(ox, oy): Location) -> Location {
		self.offset(-ox, -oy)
	}
	pub fn offset_in_direction(self, dir: Direction) -> Location {
		let (ox, oy) = dir.to_offset();
		self.offset(ox, oy)
	}

	/// Manhattan distance
	pub fn distance(self, Location(ox, oy): Location) -> i32 {
		let Location(sx, sy) = self;
		(sx-ox).abs() + (sy-oy).abs()
	}
}




#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
	North, East, South, West
}

impl Direction {
	pub fn iter_all() -> impl Iterator<Item=Direction> { (0..4).map(Into::into) }

	pub fn opposite(self) -> Direction {
		use Direction::*;
		match self {
			North => South,
			East => West,
			South => North,
			West => East,
		}
	}

	pub fn to_offset(self) -> (i32, i32) {
		match self {
			Direction::North => ( 0, 1),
			Direction::East  => ( 1, 0),
			Direction::South => ( 0,-1),
			Direction::West  => (-1, 0),
		}
	}
}

impl From<usize> for Direction {
	fn from(o: usize) -> Direction {
		match o {
			0 => Direction::North,
			1 => Direction::East,
			2 => Direction::South,
			3 => Direction::West,
			_ => panic!("Invalid direction")
		}
	}
}


impl Distribution<Direction> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
		Direction::from(rng.gen_range(0, 4))
	}
}


impl fmt::Display for Direction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Direction::North => write!(f, "North"),
			Direction::East  => write!(f, "East"),
			Direction::South => write!(f, "South"),
			Direction::West  => write!(f, "West"),
		}
	}
}







/// Inclusive bounds
#[derive(Debug, Copy, Clone)]
pub struct Bounds {
	pub min: Location,
	pub max: Location,
}

impl Bounds {
	pub fn empty() -> Bounds {
		Bounds {
			min: Location(std::i32::MAX, std::i32::MAX),
			max: Location(std::i32::MIN, std::i32::MIN),
		}
	}

	// pub fn is_valid(&self) -> bool {
	// 	self.min.0 <= self.max.0 && self.min.1 <= self.max.1
	// }

	pub fn contains(self, Location(x, y): Location) -> bool {
		self.min.0 <= x && x <= self.max.0
		&& self.min.1 <= y && y <= self.max.1
	}

	pub fn include(self, Location(x, y): Location) -> Bounds {
		Bounds {
			min: Location(self.min.0.min(x), self.min.1.min(y)),
			max: Location(self.max.0.max(x), self.max.1.max(y)),
		}
	}

	pub fn expand(self, x_amount: i32, y_amount: i32) -> Bounds {
		Bounds {
			min: Location(self.min.0 - x_amount, self.min.1 - y_amount),
			max: Location(self.max.0 + x_amount, self.max.1 + y_amount),
		}		
	}

	pub fn size(self) -> (i32, i32) {
		// +1 because Bounds is inclusive
		((self.max.0 - self.min.0 + 1).max(0), (self.max.1 - self.min.1 + 1).max(0))
	}
}