use crate::prelude::*;
use super::util::*;
use super::gfx;
use super::gfx::vertex::ColorVertex;
use super::gfx::mesh_builder::MeshBuilder;

use crate::gamestate::GameState;
use crate::task::Promise;

use super::animation::{AnimationQueue, AnimationContext};

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct PlayerView {
	mb: ColorMeshBuilder,
	pos: Vec3,
	animation_queue: AnimationQueue<PlayerView>,
}

impl PlayerView {
	pub fn new(gfx: &mut gfx::Gfx) -> PlayerView {
		PlayerView {
			mb: ColorMeshBuilder::new(gfx.core().new_mesh()),
			pos: Vec3::zero(),
			animation_queue: AnimationQueue::new(),
		}
	}

	pub fn update(&mut self, gfx: &mut gfx::Gfx, gamestate: &GameState) {
		for (f, p) in self.animation_queue.get() {
			f(self);
			p.fulfill(());
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
		let from = self.pos.to_xz();
		let to = location_to_world(to);

		self.animation_queue.start(move |ctx| async move {
			let mut phase = 0.0;

			while phase < 1.0 {
				phase += 0.015;

				let pos = phase.ease_quad_inout(from, to);
				let hop = (phase * 4.0 * PI).sin().abs() * 0.1;
				let new_pos = pos.to_x0z() + Vec3::from_y(hop);

				ctx.run(move |v| v.pos = new_pos).await;
			}

			ctx.run(move |v| v.pos = to.to_x0z()).await;

			promise.fulfill(());
		});
	}
}
