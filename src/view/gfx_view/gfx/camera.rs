use crate::prelude::*;
use crate::view::gfx_view::animation::AnimationQueue;

/// An orbit camera
pub struct Camera {
	view_size: f32,
	view_dist: f32,
	
	position: Vec3,
	yaw: f32,
	pitch: f32,

	// Cached values for speed and profit
	projection_view: Mat4,
	inv_projection_view: Mat4,

	orientation: Quat,

	// Animation stuff
	animation_queue: AnimationQueue<Camera>,
	timer: f32,
}


impl Camera {
	pub fn new() -> Self {
		Self {
			view_size: 1.2,
			view_dist: 3.0,

			position: Vec3::zero(),
			yaw: PI/8.0,
			pitch: -PI/6.0,

			projection_view: Mat4::ident(),
			inv_projection_view: Mat4::ident(),
			orientation: Quat::ident(),

			animation_queue: AnimationQueue::new(),
			timer: 0.0,
		}
	}

	pub fn update(&mut self, aspect: f32) {
		for (f, p) in self.animation_queue.get() {
			f(self);
			p.fulfill(());
		}

		self.timer += 1.0/60.0;

		let projection = Mat4::ortho_aspect(self.view_size, aspect, -100.0, 200.0);
		self.orientation = Quat::new(Vec3::from_y(1.0), self.yaw + self.timer.sin() as f32*0.02)
			* Quat::new(Vec3::from_x(1.0), self.pitch);

		let translation = Mat4::translate(-self.position + self.orientation.forward() * self.view_dist);
		self.projection_view = projection * self.orientation.conjugate().to_mat4() * translation;
		self.inv_projection_view = self.projection_view.inverse();
	}

	pub fn forward(&self) -> Vec3 {
		self.orientation.forward()
	}

	pub fn projection_view(&self) -> Mat4 {
		self.projection_view
	}

	pub fn inverse_projection_view(&self) -> Mat4 {
		self.inv_projection_view
	}


	pub fn start_move_to(&mut self, target: Vec3) {
		let start = self.position;
		let end = target;

		self.animation_queue.start(move |ctx| async move {
			let mut phase = 0.0;

			while phase < 1.0 {
				phase += 0.9/60.0;

				let new_pos = phase.ease_exp_out(start, end);
				ctx.run(move |camera| camera.position = new_pos).await;
			}

			ctx.run(move |camera| camera.position = end).await;
		});
	}

	pub fn start_zoom_to(&mut self, target: f32) {
		let start = self.view_size;
		let end = target;

		self.animation_queue.start(move |ctx| async move {
			let mut phase = 0.0;

			while phase < 1.0 {
				phase += 2.0/60.0;

				let size = phase.ease_quad_out(start, end);
				ctx.run(move |camera| camera.view_size = size).await;
			}

			ctx.run(move |camera| camera.view_size = end).await;
		});
	}

	pub fn start_rotate_to(&mut self, yaw: f32, pitch: f32) {
		let start = (self.yaw, self.pitch);
		let end = (yaw, pitch);

		self.animation_queue.start(move |ctx| async move {
			let mut phase = 0.0;

			while phase < 1.0 {
				phase += 2.0/60.0;

				let yaw = phase.ease_quad_out(start.0, end.0);
				let pitch = phase.ease_quad_out(start.1, end.1);
				ctx.run(move |camera| {
					camera.yaw = yaw;
					camera.pitch = pitch;
				}).await;
			}

			ctx.run(move |camera| {
				camera.yaw = yaw;
				camera.pitch = pitch;
			}).await;
		});
	}
}