use crate::prelude::*;
use super::gfx;
use super::vertex::{ColorVertex, Vertex};

use crate::game_state::GameState;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct MapView {
	mesh: gfx::MeshID,
	mb: ColorMeshBuilder,
}

impl MapView {
	pub fn new(gfx: &mut gfx::Gfx) -> MapView {
		MapView {
			mesh: gfx.new_mesh(ColorVertex::descriptor()),
			mb: ColorMeshBuilder::new(),
		}
	}

	pub fn render(&mut self, gfx: &mut gfx::Gfx, game_state: &GameState) {
		self.mb.clear();
		build_map(&mut self.mb, game_state);

		gfx.update_mesh(self.mesh, &self.mb.vs, &self.mb.es);
		gfx.draw_mesh(self.mesh);
	}
}


fn build_square(mb: &mut ColorMeshBuilder, pos: Vec2, size: f32, color: Color) {
	let color = color.into();
	let size = size/2.0;

	let vs = [
		ColorVertex::new((Vec2::new(-size,-size) + pos).extend(0.0), color),
		ColorVertex::new((Vec2::new(-size, size) + pos).extend(0.0), color),
		ColorVertex::new((Vec2::new( size, size) + pos).extend(0.0), color),
		ColorVertex::new((Vec2::new( size,-size) + pos).extend(0.0), color),
	];

	mb.add_quad(&vs);
}

fn build_map(mb: &mut ColorMeshBuilder, game_state: &GameState) {
	let location_to_vec = |Location(x, y): Location| -> Vec2 {
		Vec2i::new(x, y).to_vec2()*0.2
	};

	let room_color = Color::grey(0.2);
	let visited_room_color = Color::grey(0.4);
	let player_color = Color::rgb(0.5, 0.2, 0.2);

	for (location, _) in game_state.map.iter() {
		let visited = game_state.map.visited(location);
		let color = if visited { visited_room_color } else { room_color };

		build_square(mb, location_to_vec(location), 0.05, color);
	}

	build_square(mb, location_to_vec(game_state.player.location), 0.07, player_color);
}




struct MeshBuilder<V: Vertex> {
	vs: Vec<V>,
	es: Vec<u16>,
}

impl<V: Vertex> MeshBuilder<V> {
	fn new() -> MeshBuilder<V> {
		MeshBuilder {
			vs: Vec::new(),
			es: Vec::new(),
		}
	}

	fn clear(&mut self) {
		self.vs.clear();
		self.es.clear();
	}

	fn add_geometry<I, Item>(&mut self, verts: &[V], indices: I) where I: IntoIterator<Item=Item>, Item: IntoIndex {
		let start = self.vs.len();
		if start >= 0xffff {
			panic!("Too many verts!");
		}

		self.vs.extend_from_slice(verts);
		self.es.extend(indices.into_iter().map(|i| i.into_index() + start as u16));
	}

	fn add_quad(&mut self, verts: &[V]) {
		self.add_geometry(verts, &[0, 1, 2, 0, 2, 3]);
	}

	fn add_tri_fan(&mut self, vs: &[V]) {
		assert!(vs.len() >= 3);

		let indices = (1..vs.len()-1)
			.flat_map(|i| {
				let i = i as u16;
				let is = [0, i, i+1];
				(0..3).map(move |i| is[i])
			});

		self.add_geometry(vs, indices);
	}

	fn add_tri_strip(&mut self, vs: &[V]) {
		assert!(vs.len() >= 3);

		let indices = (0..vs.len()-2)
			.flat_map(|i| (0..3).map(move |offset| i as u16 + offset));

		self.add_geometry(vs, indices);
	}
}



pub trait IntoIndex {
	fn into_index(self) -> u16;
}

impl IntoIndex for u16 {
	fn into_index(self) -> u16 { self }
}

impl<'a> IntoIndex for &'a u16 {
	fn into_index(self) -> u16 { *self }
}