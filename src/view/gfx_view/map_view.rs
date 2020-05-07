use crate::prelude::*;
use super::gfx;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;

use crate::gamestate::GameState;

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

		build_map(&mut self.mb, gamestate);

		gfx.update_mesh_from(&self.mb);
		gfx.draw_mesh(self.mb.mesh_id);
	}
}


fn build_square(mb: &mut ColorMeshBuilder, pos: Vec2, size: f32, color: Color) {
	let color = color.into();
	let size = size/2.0;

	let vs = [
		ColorVertex::new((Vec2::new(-size,-size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new(-size, size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new( size, size) + pos).to_x0z(), color),
		ColorVertex::new((Vec2::new( size,-size) + pos).to_x0z(), color),
	];

	mb.add_quad(&vs);
}

fn build_map(mb: &mut ColorMeshBuilder, gamestate: &GameState) {
	let location_to_vec = |Location(x, y): Location| -> Vec2 {
		Vec2i::new(x, -y).to_vec2()*2.0
	};

	let room_color = Color::grey(0.2);
	let visited_room_color = Color::grey(0.4);
	let player_color = Color::rgb(0.5, 0.2, 0.2);

	build_square(mb, Vec2::zero(), 1.2, Color::white());

	for (location, _) in gamestate.map.iter() {
		let visited = gamestate.map.visited(location);
		let color = if visited { visited_room_color } else { room_color };

		build_square(mb, location_to_vec(location), 1.0, color);
	}

	build_square(mb, location_to_vec(gamestate.player.location), 0.4, player_color);
}

