use crate::prelude::*;
use super::util::*;
use super::gfx;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;

use crate::gamestate::GameState;
use crate::room::Room;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct MapView {
	mb: ColorMeshBuilder,
}

impl MapView {
	pub fn new(gfx: &mut gfx::Gfx) -> MapView {
		MapView {
			mb: ColorMeshBuilder::new(gfx.new_mesh()),
		}
	}

	pub fn render(&mut self, gfx: &mut gfx::Gfx, gamestate: &GameState) {
		self.mb.clear();

		build_map(&mut self.mb, &gamestate.map);

		gfx.update_mesh_from(&self.mb);
		gfx.draw_mesh(self.mb.mesh_id);
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