use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui::Context as UiContext};
use super::click_region::*;

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

	pub fn render(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
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

	pub fn process_mouse_event(&mut self, event: ClickRegionEvent) -> Option<PlayerCommand> {
		if self.player_can_move() {
			event.update_multiple(&mut self.door_views)
		} else {
			None
		}
	}
}


fn build_room(gfx: &mut Gfx, pos: Vec2, room: Room, visited: bool) {
	let room_color = Color::grey(0.2);
	let visited_room_color = Color::grey(0.4);
	let color = if visited { visited_room_color } else { room_color };

	gfx.ui().quad(pos.to_x0z(), Vec2::splat(1.0), color, UiContext::World);

	for dir in room.iter_neighbor_directions().map(direction_to_offset) {
		let pos = pos + dir * 0.5;
		let size = dir + dir.perp() * 0.4;
		gfx.ui().quad(pos.to_x0z(), size, color, UiContext::World);
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





enum DoorViewState {
	Hidden,
	Idle,
	Hovering(f32),
	HoverExit(f32),
	Clicked(f32),
}


struct DoorView {
	pos: Vec2,
	dir: Direction,
	state: DoorViewState,
}

impl DoorView {
	fn new(pos: Vec2, dir: Direction) -> Self {
		Self {
			pos, dir,
			state: DoorViewState::Idle,
		}
	}

	fn set_enabled(&mut self, enabled: bool) {
		self.state = if enabled {
			DoorViewState::Idle
		} else {
			DoorViewState::Hidden
		};
	}

	fn update(&mut self, gfx: &mut Gfx) {
		use DoorViewState::*;

		let hover_fade_rate = 0.1;
		let click_fade_rate = 0.1;

		self.state = match self.state {
			Hidden => return,

			Idle => Idle,

			Hovering(v) => Hovering((v+hover_fade_rate).min(1.0)),

			HoverExit(v) => if v > 0.0 {
				HoverExit((v-hover_fade_rate).max(0.0))
			} else {
				Idle
			},

			Clicked(v) => if v > 0.0 { Clicked((v-click_fade_rate).max(0.0)) } else { Hovering(1.0) },
		};

		let base_col = Color::rgba(1.0, 0.4, 0.5, 0.2);
		let hover_col = Color::rgba(1.0, 0.4, 0.5, 1.0);
		let click_col = Color::white();

		let (size, color) = match self.state {
			Hidden => unreachable!(),
			Idle => (0.6, base_col),
			Hovering(v) | HoverExit(v) => (0.6 + v*0.02, v.ease_linear(base_col, hover_col)),
			Clicked(v) => (0.6 + 0.02, v.ease_back_in(hover_col, click_col)),
		};

		let shadow_color = Color::grey_a(0.1, 0.1);

		let shadow_pos = self.position().to_x0z() + Vec3::from_y(0.005);
		let main_pos = shadow_pos + Vec3::from_y(0.05);

		gfx.ui().arrow(self.dir, shadow_pos, size, shadow_color, UiContext::World);
		gfx.ui().arrow(self.dir, main_pos, size, color, UiContext::World);
	}
}

impl ClickRegion for DoorView {
	type ClickResponse = PlayerCommand;

	fn context(&self) -> ClickRegionContext { ClickRegionContext::Ground }

	fn position(&self) -> Vec2 { self.pos + direction_to_offset(self.dir) * 0.75 }
	fn size(&self) -> Vec2 {
		if matches!(self.state, DoorViewState::Hidden) {
			Vec2::zero()
		} else {
			Vec2::splat(0.6)
		}
	}

	fn while_mouse_over(&mut self) {
		use DoorViewState::*;

		self.state = match self.state {
			Idle => Hovering(0.0),
			HoverExit(v) => Hovering(v),
			_ => return,
		};
	}

	fn while_mouse_not_over(&mut self) {
		use DoorViewState::*;

		self.state = match self.state {
			Hovering(v) => HoverExit(v),
			_ => return,
		};
	}

	fn on_click(&mut self) -> PlayerCommand {
		use crate::controller::main::PlayerCommand::*;

		self.state = DoorViewState::Clicked(1.0);
		match self.dir {
			Direction::North => PlayerCommand::from(GoNorth),
			Direction::East => PlayerCommand::from(GoEast),
			Direction::South => PlayerCommand::from(GoSouth),
			Direction::West => PlayerCommand::from(GoWest),
		}
	}
}

