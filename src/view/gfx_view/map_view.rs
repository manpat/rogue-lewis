use crate::prelude::*;
use super::util::*;
use super::gfx;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;
use super::click_region::*;

use crate::task::PlayerCommand;
use crate::gamestate::GameState;
use crate::room::Room;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct MapView {
	mb: ColorMeshBuilder,
	door_views: [DoorView; 4],
}

impl MapView {
	pub fn new(gfx: &mut gfx::Gfx) -> MapView {
		MapView {
			mb: ColorMeshBuilder::new(gfx.new_mesh()),

			door_views: [
				DoorView::new(Vec2::zero(), Direction::North),
				DoorView::new(Vec2::zero(), Direction::East),
				DoorView::new(Vec2::zero(), Direction::South),
				DoorView::new(Vec2::zero(), Direction::West),
			],
		}
	}

	pub fn init(&mut self, gamestate: &GameState) {
		self.on_player_move(gamestate);
	}

	pub fn render(&mut self, gfx: &mut gfx::Gfx, gamestate: &GameState) {
		self.mb.clear();

		build_map(&mut self.mb, &gamestate.map);

		for view in self.door_views.iter_mut() {
			view.update(&mut self.mb);
		}

		gfx.update_mesh_from(&self.mb);
		gfx.draw_mesh(self.mb.mesh_id);
	}

	pub fn on_player_move(&mut self, gamestate: &GameState) {
		let world_pos = location_to_world(gamestate.player.location);
		let room = gamestate.map.get(gamestate.player.location).unwrap();

		for view in self.door_views.iter_mut() {
			view.set_enabled(room.door(view.dir));
			view.pos = world_pos;
		}
	}

	pub fn process_mouse_event(&mut self, event: ClickRegionEvent) -> Option<PlayerCommand> {
		event.update_multiple(&mut self.door_views)
	}
}


fn build_room(mb: &mut ColorMeshBuilder, pos: Vec2, room: Room, visited: bool) {
	let room_color = Color::grey(0.2);
	let visited_room_color = Color::grey(0.4);
	let color = if visited { visited_room_color } else { room_color };

	let color = color.into();
	let size = 0.5;

	mb.add_quad(&[
		ColorVertex::new((Vec2::new(-size,-size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new(-size, size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new( size, size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new( size,-size) + pos).to_x0z(), color),
	]);


	for dir in room.iter_neighbor_directions().map(direction_to_offset) {
		let pos = pos + dir * 0.5;

		let minor = dir.perp() * 0.2;
		let major = dir * 0.5;

		mb.add_quad(&[
			ColorVertex::new((pos - minor - major).to_x0z(), color),
			ColorVertex::new((pos + minor - major).to_x0z(), color),
			ColorVertex::new((pos + minor + major).to_x0z(), color),
			ColorVertex::new((pos - minor + major).to_x0z(), color),
		]);
	}
}

fn build_map(mb: &mut ColorMeshBuilder, map: &crate::map::Map) {
	for (location, room) in map.iter() {
		let visited = map.visited(location);
		build_room(mb, location_to_world(location), room, visited);
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

	fn update(&mut self, mb: &mut ColorMeshBuilder) {
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

		let pos = self.position().to_x0z();
		let color = color.into();
		let shadow_color = Color::grey_a(0.1, 0.1).into();

		let offset = direction_to_offset(self.dir);

		let major = offset.to_x0z() * (size/2.0);
		let minor = offset.perp().to_x0z() * (size/2.0);
		let hover_dist = 0.05;

		// Draw arrow shadow
		mb.add_tri_fan(&[
			ColorVertex::new(pos + major, shadow_color),
			ColorVertex::new(pos - major + minor, shadow_color),
			ColorVertex::new(pos - major * 0.5, shadow_color),
			ColorVertex::new(pos - major - minor, shadow_color),
		]);

		// Draw main arrow
		mb.add_tri_fan(&[
			ColorVertex::new(pos + Vec3::from_y(hover_dist) + major, color),
			ColorVertex::new(pos + Vec3::from_y(hover_dist) - major + minor, color),
			ColorVertex::new(pos + Vec3::from_y(hover_dist) - major * 0.5, color),
			ColorVertex::new(pos + Vec3::from_y(hover_dist) - major - minor, color),
		]);
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

