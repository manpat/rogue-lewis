use crate::prelude::*;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;
use crate::task::PlayerCommand;
use crate::view::gfx_view::util;


pub type UiVertex = ColorVertex;

#[derive(Copy, Clone, Debug)]
pub enum Context {
	Screen, Ground,
}


pub struct Ui {
	dumb_quads: Vec<DumbQuad>,
	dumb_arrows: Vec<DumbArrow>,

	camera_forward: Vec3,
	camera_near_pos: Vec3,
	screen_mouse: Vec2,

	commands: Vec<PlayerCommand>,
	click_occured: bool,
}

impl Ui {
	pub fn new() -> Self {
		Self {
			dumb_quads: Vec::new(),
			dumb_arrows: Vec::new(),

			camera_forward: Vec3::zero(),
			camera_near_pos: Vec3::zero(),
			screen_mouse: Vec2::zero(),

			commands: Vec::new(),
			click_occured: false,
		}
	}

	pub fn clear(&mut self) {
		self.dumb_quads.clear();
		self.dumb_arrows.clear();
	}

	pub fn update(&mut self, camera_forward: Vec3, camera_near_pos: Vec3) {
		self.camera_forward = camera_forward;
		self.camera_near_pos = camera_near_pos;
	}

	pub fn drain_commands(&mut self) -> Vec<PlayerCommand> {
		std::mem::replace(&mut self.commands, Vec::new())
	}

	pub fn on_mouse_move(&mut self, screen_mouse: Vec2) {
		self.screen_mouse = screen_mouse;
	}

	pub fn clear_click_state(&mut self) {
		self.click_occured = false;
	}

	pub fn on_mouse_click(&mut self) {
		self.click_occured = true;
	}

	pub(super) fn build_world_space(&mut self, mb: &mut MeshBuilder<UiVertex>) {
		for &DumbQuad {pos, size, color, ..} in 
			self.dumb_quads.iter().filter(|q| matches!(q.context, Context::Ground))
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
			self.dumb_arrows.iter().filter(|q| matches!(q.context, Context::Ground))
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


	pub fn update_interact_region<F, P>(&mut self, hoverable: &mut Hoverable, region: &Region, on_click: F)
		where F: FnOnce() -> P, P: Into<PlayerCommand>
	{
		hoverable.update();

		if region.includes(self) {
			hoverable.begin_hover();

			if self.click_occured {
				hoverable.click();
				self.commands.push(on_click().into());

				// Don't click more than one thing
				self.click_occured = false;
			}

		} else {
			hoverable.end_hover();
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



#[derive(Copy, Clone, Debug)]
pub struct Region {
	pub pos: Vec3,
	pub size: Vec2,
	pub context: Context,
}

impl Region {
	pub fn new_ground(pos: Vec3, size: Vec2) -> Region {
		Region {
			pos, size,
			context: Context::Ground,
		}
	}

	pub fn new_screen(pos: Vec3, size: Vec2) -> Region {
		Region {
			pos, size,
			context: Context::Screen,
		}
	}

	fn includes(&self, ui: &Ui) -> bool {
		let (self_pos, mouse_pos) = match self.context {
			Context::Screen => (self.pos.to_xy(), ui.screen_mouse),
			Context::Ground => {
				let plane_pos = Vec3::from_y(self.pos.y);
				let plane_normal = Vec3::from_y(1.0);

				let res = util::intersect_plane(
					plane_pos, plane_normal,
					ui.camera_near_pos, ui.camera_forward
				);

				match res {
					Some(pos) => (self.pos.to_xz(), pos.to_xz()),
					None => return false,
				}
			}
		};

		let diff = self_pos - mouse_pos;
		let extent = self.size / 2.0;

		diff.x.abs() < extent.x && diff.y.abs() < extent.y
	}
}



#[derive(Copy, Clone, Debug)]
pub enum HoverState {
	Idle,
	HoverEnter(f32),
	Hovering,
	HoverExit(f32),
	Clicked(f32),
}

pub struct Hoverable {
	state: HoverState,
	hover_fade_rate: f32,
	click_fade_rate: f32,
}

impl Hoverable {
	pub fn new(hover_fade_rate: f32, click_fade_rate: f32) -> Hoverable {
		Hoverable {
			state: HoverState::Idle,
			hover_fade_rate,
			click_fade_rate,
		}
	}

	pub fn update(&mut self) {
		use HoverState::*;

		self.state = match self.state {
			HoverEnter(v) => if v < 1.0 {
				HoverEnter((v+self.hover_fade_rate).min(1.0))
			} else {
				Hovering
			},

			HoverExit(v) => if v > 0.0 {
				HoverExit((v-self.hover_fade_rate).max(0.0))
			} else {
				Idle
			},

			Clicked(v) => if v > 0.0 {
				Clicked((v-self.click_fade_rate).max(0.0))
			} else {
				HoverEnter(1.0)
			},

			x => x,
		};
	}

	pub fn reset(&mut self) {
		self.state = HoverState::Idle;
	}

	pub fn begin_hover(&mut self) {
		use HoverState::*;

		self.state = match self.state {
			Idle => HoverEnter(0.0),
			HoverExit(v) => HoverEnter(v),
			_ => return,
		};
	}

	pub fn end_hover(&mut self) {
		use HoverState::*;

		self.state = match self.state {
			HoverEnter(v) => HoverExit(v),
			Hovering => HoverExit(1.0),
			_ => return,
		};
	}

	pub fn click(&mut self) {
		self.state = HoverState::Clicked(1.0);
	}

	pub fn state(&self) -> HoverState { self.state }
}


impl Default for Hoverable {
	fn default() -> Hoverable {
		Hoverable::new(0.1, 0.1)
	}
}