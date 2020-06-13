use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui};

use crate::task::{PlayerCommand, ControllerMode, Promise};
use crate::gamestate::GameState;
use crate::room::Room;

pub struct MapView {
	door_views: [DoorView; 4],

	player_move_in_progress: bool,
	full_map_view_requested: bool,
	in_main_mode: bool,

	mode_change: Option<ControllerMode>,
	full_map_promise: Option<Promise<()>>,

	interact_hoverable: ui::Hoverable,
	close_map_hoverable: ui::Hoverable,
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
			full_map_view_requested: false,
			in_main_mode: true,

			mode_change: None,
			full_map_promise: None,

			interact_hoverable: Default::default(),
			close_map_hoverable: Default::default(),
		}
	}

	pub fn init(&mut self, gamestate: &GameState) {
		self.on_player_move(gamestate);
	}

	pub fn update(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if self.full_map_view_requested {
			self.full_map_view_requested = false;
			gfx.camera.start_zoom_to(10.0);
			gfx.camera.start_rotate_to(PI/30.0, -PI/3.0);
		}

		match self.mode_change.take() {
			Some(ControllerMode::Main) => {
				gfx.camera.start_zoom_to(1.2);
				gfx.camera.start_rotate_to(PI/8.0, -PI/6.0);
			}

			Some(ControllerMode::Battle)
			| Some(ControllerMode::Merchant) => {
				gfx.camera.start_zoom_to(0.9);
				gfx.camera.start_rotate_to(PI/5.0, -PI/9.0);
			}

			None => {}
		}

		build_map(gfx, &gamestate.map);

		if self.player_can_move() {
			for view in self.door_views.iter_mut() {
				view.update(gfx);
			}

			let player_loc = gamestate.player.location;
			let current_room = gamestate.map.get(player_loc).unwrap();
			if current_room.has_interactable() {
				use crate::controller::main::PlayerCommand::*;

				let size = Vec2::splat(0.2);
				let pos = location_to_world(player_loc).to_x0z() + Vec3::new(0.7, 0.01, 0.7);

				let region = ui::Region::new_ground(pos, size);

				let clicked = gfx.ui.update_interact_region(
					&mut self.interact_hoverable,
					&region,
					|| Interact
				);

				let color = ui::palette().map.color(self.interact_hoverable.state());
				gfx.ui.quad(region, color);
			}
		}

		if self.full_map_promise.is_some() {
			let size = Vec2::splat(0.2);
			let pos = Vec3::new(0.11, -0.11, 0.0);

			let region = ui::Region::new(pos, size, ui::Context::ScreenTopLeft);

			let clicked = gfx.ui.update_immediate_interact_region(
				&mut self.close_map_hoverable,
				&region
			);

			let color = ui::palette().map.color(self.close_map_hoverable.state());
			gfx.ui.quad(region, color);

			if clicked {
				gfx.camera.start_zoom_rotate_to_default();
				self.full_map_promise.take()
					.unwrap()
					.fulfill(());
			}
		}
	}

	fn player_can_move(&self) -> bool {
		self.in_main_mode && !self.player_move_in_progress
			&& self.full_map_promise.is_none()
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.in_main_mode = matches!(mode, ControllerMode::Main);
		self.mode_change = Some(mode);
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

	pub fn show_map(&mut self, promise: Promise<()>) {
		assert!(self.full_map_promise.is_none());
		self.full_map_promise = Some(promise);
		self.full_map_view_requested = true;
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

	gfx.ui.quad((pos.to_x0z(), Vec2::splat(1.0), ui::Context::Ground), color);

	for dir in room.iter_neighbor_directions().map(direction_to_offset) {
		let pos = pos + dir * 0.5;
		let size = dir + dir.perp() * 0.4;
		gfx.ui.quad((pos.to_x0z(), size, ui::Context::Ground), color);
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

		gfx.ui.update_interact_region(
			&mut self.hoverable,
			&region,
			|| match dir {
				Direction::North => GoNorth,
				Direction::East => GoEast,
				Direction::South => GoSouth,
				Direction::West => GoWest,
			}
		);

		use HoverState::*;
		let size = match self.hoverable.state() {
			Idle => 0.6,
			HoverEnter(v) | HoverExit(v) => 0.6 + v*0.02,
			Hovering => 0.62,
			Clicked(v) => 0.6 + 0.02,
		};

		let size = Vec2::splat(size);
		let color = ui::palette().movement.color(self.hoverable.state());

		let shadow_color = Color::grey_a(0.1, 0.1);

		let shadow_pos = self.position().to_x0z() + Vec3::from_y(0.005);
		let main_pos = shadow_pos + Vec3::from_y(0.05);

		gfx.ui.arrow((shadow_pos, size, ui::Context::Ground), self.dir, shadow_color);
		gfx.ui.arrow((main_pos, size, ui::Context::Ground), self.dir, color);
	}
}