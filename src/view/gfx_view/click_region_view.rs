use crate::prelude::*;
use crate::task::{PlayerCommand, ControllerMode};
use super::gfx::{Gfx, MeshID};
use super::mesh_builder::MeshBuilder;
use super::vertex::ColorVertex;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct ClickRegionView {
	mb: ColorMeshBuilder,
	regions: Vec<ClickRegion>,
}


impl ClickRegionView {
	pub fn new(gfx: &mut Gfx) -> Self {
		Self {
			mb: ColorMeshBuilder::new(gfx.new_mesh()),
		}
	}

	pub fn render(&mut self, gfx: &mut Gfx) {
		// self.mb.clear();

		// build_square(&mut self.mb, Vec2::zero(), 0.3, Color::grey_a(0.8, 0.3));

		gfx.update_mesh_from(&self.mb);
		gfx.draw_mesh(self.mb.mesh_id);
	}

	pub fn process_click(&mut self, world: Vec2, ctl_mode: ControllerMode) -> Option<PlayerCommand> {
		build_square(&mut self.mb, world, 0.5, Color::rgba(0.5, 0.8, 0.4, 0.3));

		None
	}

	pub fn process_hover(&mut self, world: Vec2) {
		build_square(&mut self.mb, world, 0.1, Color::rgba(0.5, 0.4, 0.8, 0.03));
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


struct ClickRegion {
	pos: Vec2,
	size: f32,

	hover_state: f32, // [0, 1]
}