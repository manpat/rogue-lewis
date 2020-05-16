use crate::prelude::*;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;


pub type UiVertex = ColorVertex;


pub enum Context {
	World, Screen,
}


pub struct Ui {
	dumb_quads: Vec<DumbQuad>,
	dumb_arrows: Vec<DumbArrow>,
}

impl Ui {
	pub fn new() -> Self {
		Self {
			dumb_quads: Vec::new(),
			dumb_arrows: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.dumb_quads.clear();
		self.dumb_arrows.clear();
	}

	pub(super) fn build_world_space(&mut self, mb: &mut MeshBuilder<UiVertex>) {
		for &DumbQuad {pos, size, color, ..} in 
			self.dumb_quads.iter().filter(|q| matches!(q.context, Context::World))
		{
			let size = size / 2.0;
			let color = color.into();

			mb.add_quad(&[
				UiVertex::new(Vec3::new(-size.x, 0.0, -size.y) + pos, color),
				UiVertex::new(Vec3::new(-size.x, 0.0,  size.y) + pos, color),
				UiVertex::new(Vec3::new( size.x, 0.0,  size.y) + pos, color),
				UiVertex::new(Vec3::new( size.x, 0.0, -size.y) + pos, color),
			]);
		}

		for &DumbArrow {direction, pos, size, color, ..} in 
			self.dumb_arrows.iter().filter(|q| matches!(q.context, Context::World))
		{
			let color = color.into();
			let offset = direction_to_offset(direction);

			let major = offset.to_x0z() * (size/2.0);
			let minor = offset.perp().to_x0z() * (size/2.0);

			// Draw arrow shadow
			mb.add_tri_fan(&[
				UiVertex::new(pos + major, color),
				UiVertex::new(pos - major + minor, color),
				UiVertex::new(pos - major * 0.5, color),
				UiVertex::new(pos - major - minor, color),
			]);
		}
	}

	pub(super) fn build_screen_space(&mut self, mb: &mut MeshBuilder<UiVertex>) {
		for &DumbQuad {pos, size, color, ..} in 
			self.dumb_quads.iter().filter(|q| matches!(q.context, Context::Screen))
		{
			let size = size / 2.0;
			let color = color.into();

			mb.add_quad(&[
				UiVertex::new(Vec3::new(-size.x, -size.y, 0.0) + pos, color),
				UiVertex::new(Vec3::new(-size.x,  size.y, 0.0) + pos, color),
				UiVertex::new(Vec3::new( size.x,  size.y, 0.0) + pos, color),
				UiVertex::new(Vec3::new( size.x, -size.y, 0.0) + pos, color),
			]);
		}
	}


	pub fn quad(&mut self, pos: Vec3, size: Vec2, color: Color, context: Context) {
		self.dumb_quads.push(DumbQuad { context, pos, size, color });
	}

	pub fn arrow(&mut self, direction: Direction, pos: Vec3, size: f32, color: Color, context: Context) {
		self.dumb_arrows.push(DumbArrow { context, direction, pos, size, color });
	}
}



struct DumbQuad {
	context: Context,
	pos: Vec3,
	size: Vec2,
	color: Color,
}

struct DumbArrow {
	context: Context,
	direction: Direction,
	pos: Vec3,
	size: f32,
	color: Color,
}

fn direction_to_offset(d: Direction) -> Vec2 {
	match d {
		Direction::North => Vec2::from_y(-1.0),
		Direction::South => Vec2::from_y( 1.0),
		Direction::East => Vec2::from_x( 1.0),
		Direction::West => Vec2::from_x(-1.0),
	}
}
