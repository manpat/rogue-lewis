use crate::prelude::*;
use crate::rendering::RenderBuffer;
use crate::map::Map;
use crate::room::Room;

#[derive(Debug)]
pub struct GameState {
	pub map: Map,
	pub player_location: Location,
}


impl GameState {
	pub fn new() -> GameState {
		GameState {
			map: Map::new(),
			player_location: Location(0, 0),
		}
	}
}