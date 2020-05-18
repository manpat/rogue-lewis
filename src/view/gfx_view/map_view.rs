use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui};

use crate::task::{PlayerCommand, ControllerMode};
use crate::gamestate::GameState;
use crate::room::Room;

pub struct MapView {
	door_views: [DoorView; 4],

	player_move_in_progress: bool,
	in_main_mode: bool,
}

impl MapView {
	pub fn new(gfx: &mut Gfx) -> MapView {
		MapView {
			door_views: [
				DoorView::new(Vec2::zero(), Direction::North),
				DoorView::new(Vec2::zero(), Direction::East),
				DoorView::new(Vec2::zero(), Direction::South),
				DoorView::new(Vec2::zero(), Direction::West),
			],

			player_move_in_progress: false,
			in_main_mode: true,
		}
	}

	pub fn init(&mut self, gamestate: &GameState) {
		self.on_player_move(gamestate);
	}

	pub fn update(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		build_map(gfx, &gamestate.map);

		if self.player_can_move() {
			for view in self.door_views.iter_mut() {
				view.update(gfx);
			}
		}
	}

	fn player_can_move(&self) -> bool {
		self.in_main_mode && !self.player_move_in_progress
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.in_main_mode = matches!(mode, ControllerMode::Main);
	}

	pub fn on_player_move(&mut self, gamestate: &GameState) {
		assert!(self.player_can_move());
		self.player_move_in_progress = true;

		let world_pos = location_to_world(gamestate.player.location);
		let room = gamestate.map.get(gamestate.player.location).unwrap();

		for view in self.door_views.iter_mut() {
			view.set_enabled(room.door(view.dir));
			view.pos = world_pos;
		}
	}

	// TODO: this whole deal would probably be better off as a future
	// what we're effectively waiting for is the player move animation to 
	// finish, and for encounters to run
	pub fn on_awaiting_player_command(&mut self) {
		self.player_move_in_progress = false;
	}
}


fn build_room(gfx: &mut Gfx, pos: Vec2, room: Room, visited: bool) {
	let room_color = Color::grey(0.2);
	let visited_room_color = Color::grey(0.4);
	let color = if visited { visited_room_color } else { room_color };

	gfx.ui().quad(pos.to_x0z(), Vec2::splat(1.0), color, ui::Context::Ground);

	for dir in room.iter_neighbor_directions().map(direction_to_offset) {
		let pos = pos + dir * 0.5;
		let size = dir + dir.perp() * 0.4;
		gfx.ui().quad(pos.to_x0z(), size, color, ui::Context::Ground);
	}
}

fn build_map(gfx: &mut Gfx, map: &crate::map::Map) {
	for (location, room) in map.iter() {
		let visited = map.visited(location);
		build_room(gfx, location_to_world(location), room, visited);
	}
}

fn direction_to_offset(d: Direction) -> Vec2 {
	match d {
		Direction::North => Vec2::from_y(-1.0),
		Direction::South => Vec2::from_y( 1.0),
		Direction::East => Vec2::from_x( 1.0),
		Direction::West => Vec2::from_x(-1.0),
	}
}


use super::gfx::ui::{Hoverable, HoverState};

struct DoorView {
	enabled: bool,
	pos: Vec2,
	dir: Direction,
	hoverable: Hoverable,
}

impl DoorView {
	fn new(pos: Vec2, dir: Direction) -> Self {
		Self {
			enabled: true,
			pos, dir,
			hoverable: Hoverable::new(0.1, 0.1),
		}
	}

	fn set_enabled(&mut self, enabled: bool) {
		self.enabled = enabled;
		if self.enabled {
			self.hoverable.reset();
		}
	}

	fn position(&self) -> Vec2 { self.pos + direction_to_offset(self.dir) * 0.75 }

	fn update(&mut self, gfx: &mut Gfx) {
		if !self.enabled { return }

		use crate::controller::main::PlayerCommand::*;

		let dir = self.dir;
		let region = ui::Region::new_ground(self.position().to_x0z(), Vec2::splat(0.6));

		gfx.ui().update_interact_region(
			&mut self.hoverable,
			&region,
			|| match dir {
				Direction::North => GoNorth,
				Direction::East => GoEast,
				Direction::South => GoSouth,
				Direction::West => GoWest,
			}
		);

		let base_col = Color::rgba(1.0, 0.4, 0.5, 0.2);
		let hover_col = Color::rgba(1.0, 0.4, 0.5, 1.0);
		let click_col = Color::white();

		use HoverState::*;
		let (size, color) = match self.hoverable.state() {
			Idle => (0.6, base_col),
			HoverEnter(v) | HoverExit(v) => (0.6 + v*0.02, v.ease_linear(base_col, hover_col)),
			Hovering => (0.62, hover_col),
			Clicked(v) => (0.6 + 0.02, v.ease_back_in(hover_col, click_col)),
		};

		let shadow_color = Color::grey_a(0.1, 0.1);

		let shadow_pos = self.position().to_x0z() + Vec3::from_y(0.005);
		let main_pos = shadow_pos + Vec3::from_y(0.05);

		gfx.ui().arrow(self.dir, shadow_pos, size, shadow_color, ui::Context::Ground);
		gfx.ui().arrow(self.dir, main_pos, size, color, ui::Context::Ground);
	}
}