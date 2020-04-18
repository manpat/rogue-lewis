pub mod render_buffer;
pub use render_buffer::*;

use crate::prelude::*;
use crate::game_state::{GameState, Item};
use crate::room::{Room, EncounterType};


// https://en.wikipedia.org/wiki/Box_Drawing_(Unicode_block)
// https://en.wikipedia.org/wiki/Unicode_symbols

// const EMPTY_CHAR: char = '░'; // '\u{2591}';
const EMPTY_CHAR: char = ' ';

// const PLAYER_CHAR: char = '⚉'; // '\u{2689}';
const PLAYER_CHAR: char = '⚲'; // '\u{26B2}';


pub fn render_map(state: &GameState, bounds: Bounds) -> RenderBuffer {
	let (width, height) = bounds.size();

	assert!(width > 0 && height > 0, "Map is empty!");

	let x_scale = 3;
	let y_scale = 2;

	let width = width as usize * x_scale as usize + 1;
	let height = height as usize * y_scale as usize + 1;

	let mut buffer = RenderBuffer::new(width + 2, height + 2);
	let room_to_buffer_space = move |loc: Location| {
		loc.relative_to(bounds.min).scale(x_scale, y_scale).offset(2, 2)
	};

	let player_has_map = state.player.inventory.has(Item::Map);

	buffer.fill(EMPTY_CHAR);

	for (location, room) in state.map.iter().filter(|&(l, _)| bounds.contains(l)) {
		let room_dist = state.player.location.distance(location);
		let obscured = room_dist > 2;

		let room_visited = player_has_map || state.map.visited(location);

		let buffer_location = room_to_buffer_space(location);
		buffer.write(buffer_location, block_for_room(&room, obscured, room_visited));

		for dir in room.iter_neighbor_directions() {
			let corridor_loc = buffer_location.offset_in_direction(dir);
			let target_room_loc = location.offset_in_direction(dir);
			let connected = state.map.has(target_room_loc);

			buffer.write(corridor_loc, corridor_for_direction(dir, connected, obscured));
		}
	}

	buffer.write(room_to_buffer_space(state.player.location), PLAYER_CHAR);

	buffer
}

fn corridor_for_direction(dir: Direction, connected: bool, obscured: bool) -> char {
	const SOLID_CORRIDOR: [char; 4] = ['│', '─', '│', '─'];
	// const THICK_CORRIDOR: [char; 4] = ['┃', '━', '┃', '━'];
	// const DOUBLE_CORRIDOR: [char; 4] = ['║', '═', '║', '═'];
	const OBSCURED_CORRIDOR: [char; 4] = ['┊', '┄', '┊', '┄'];

	const ARROWS: [char; 4] = ['↑', '→', '↓', '←'];

	let style = if connected {
		if obscured { OBSCURED_CORRIDOR } else { SOLID_CORRIDOR }
	} else {
		ARROWS
	};

	style[dir as usize]
}

fn block_for_room(room: &Room, obscured: bool, visited: bool) -> char {
	if !visited {
		return '□';
	}

	if room.is_exit {
		return if obscured {'◬'} else {'▲'};
	}

	if let Some(encounter) = room.encounter {
		return block_for_encounter(encounter);
	}

	if obscured {
		return '🝕';
	}

	'■'
}

fn block_for_encounter(encounter: EncounterType) -> char {
	match encounter {
		EncounterType::Food 	=> '+',
		EncounterType::Treasure => '+',
		EncounterType::Key 		=> '+',
		EncounterType::Equipment=> '+',
		
		EncounterType::Map 		=> '%',

		EncounterType::Merchant => '$',
		EncounterType::Chest 	=> 'C',
		EncounterType::Trap 	=> 'X',

		EncounterType::Monster 	=> 'M',
		EncounterType::Boss 	=> 'B',
	}
}