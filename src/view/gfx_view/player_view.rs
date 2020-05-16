use crate::prelude::*;
use super::util::*;
use super::gfx;
use super::gfx::vertex::ColorVertex;
use super::gfx::mesh_builder::MeshBuilder;

use crate::gamestate::GameState;
use crate::task::Promise;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct PlayerView {
	mb: ColorMeshBuilder,
	pos: Vec3,

	move_animation: Option<MoveAnimation>,
}

impl PlayerView {
	pub fn new(gfx: &mut gfx::Gfx) -> PlayerView {
		PlayerView {
			mb: ColorMeshBuilder::new(gfx.core().new_mesh()),
			pos: Vec3::zero(),
			move_animation: None,
		}
	}

	pub fn render(&mut self, gfx: &mut gfx::Gfx, gamestate: &GameState) {
		let anim_finished = if let Some(MoveAnimation {from, to, ref mut phase, ..}) = self.move_animation {
			*phase += 0.015;

			let pos = phase.ease_quad_inout(from, to);
			let hop = (*phase * 4.0 * PI).sin().abs() * 0.1;

			self.pos = pos.to_x0z() + Vec3::from_y(hop);

			*phase >= 1.0

		} else {
			false
		};

		if anim_finished {
			let MoveAnimation {promise, to, ..} = self.move_animation.take().unwrap();
			self.pos = to.to_x0z();
			promise.fulfill(());
		}

		self.mb.clear();

		let color = Color::white().into();
		let (w, h) = (0.15, 0.5);

		let vs = [
			ColorVertex::new(Vec3::new(-w, 0.0,  w) + self.pos, color),
			ColorVertex::new(Vec3::new(-w,   h,  w) + self.pos, color),
			ColorVertex::new(Vec3::new( w,   h, -w) + self.pos, color),
			ColorVertex::new(Vec3::new( w, 0.0, -w) + self.pos, color),
		];

		self.mb.add_quad(&vs);

		gfx.core().update_mesh_from(&self.mb);
		gfx.core().draw_mesh(self.mb.mesh_id);
	}

	pub fn on_player_move(&mut self, to: Location, promise: Promise<()>) {
		assert!(self.move_animation.is_none());

		self.move_animation = Some(MoveAnimation {
			from: self.pos.to_xz(),
			to: location_to_world(to),
			phase: 0.0,

			promise,
		});
	}
}

struct MoveAnimation {
	from: Vec2,
	to: Vec2,
	phase: f32,

	promise: Promise<()>,
}