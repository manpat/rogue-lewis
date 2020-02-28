use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Room {
	pub doors: [bool; 4],
}

impl Room {
	pub fn with_doors(doors: [bool; 4]) -> Room { Room { doors } }
	pub fn new() -> Room { Room::with_doors([false; 4]) }

	pub fn door(&self, dir: Direction) -> bool { self.doors[dir as usize] }
	pub fn set_door(&mut self, dir: Direction, open: bool) { self.doors[dir as usize] = open; }

	pub fn neighbor_directions(&self) -> impl Iterator<Item=Direction> + '_ {
		self.doors.iter().cloned()
			.enumerate()
			.filter(|&(_, door)| door)
			.map(|(idx, _)| idx.into())
	}
}