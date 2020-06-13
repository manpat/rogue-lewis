use crate::prelude::*;
use super::vertex::ColorVertex;
use super::mesh_builder::MeshBuilder;
use crate::task::PlayerCommand;
use crate::view::gfx_view::util;


pub type UiVertex = ColorVertex;

#[derive(Copy, Clone, Debug)]
pub enum Context {
	ScreenCenter,
	ScreenTopLeft,
	ScreenTopRight,
	ScreenBottomLeft,
	ScreenBottomRight,
	Ground,
}

impl Context {
	pub fn is_screen(&self) -> bool {
		use Context::*;

		matches!(
			self,
			ScreenCenter
			| ScreenTopLeft | ScreenTopRight
			| ScreenBottomLeft | ScreenBottomRight
		)
	}

	pub fn is_world(&self) -> bool {
		use Context::*;

		matches!(
			self,
			Ground
		)
	}
}


pub struct Ui {
	dumb_quads: Vec<DumbQuad>,
	dumb_arrows: Vec<DumbArrow>,

	camera_forward: Vec3,
	camera_near_pos: Vec3,
	screen_mouse: Vec2,
	aspect: f32,

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
			aspect: 1.0,

			commands: Vec::new(),
			click_occured: false,
		}
	}

	pub fn clear(&mut self) {
		self.dumb_quads.clear();
		self.dumb_arrows.clear();
	}

	pub fn update(&mut self, camera_forward: Vec3, camera_near_pos: Vec3, aspect: f32) {
		self.camera_forward = camera_forward;
		self.camera_near_pos = camera_near_pos;
		self.aspect = aspect;
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
		for &DumbQuad {region: Region{pos, size, ..}, color} in 
			self.dumb_quads.iter().filter(|q| q.region.context.is_world())
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

		for &DumbArrow {region: Region{pos, size, ..}, direction, color} in 
			self.dumb_arrows.iter().filter(|a| a.region.context.is_world())
		{
			let color = color.into();
			let offset = direction_to_offset(direction);

			let major = offset.to_x0z() * (size.y/2.0);
			let minor = offset.perp().to_x0z() * (size.x/2.0);

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
		use Context::*;

		let aspect = aspect_to_vec(self.aspect).extend(1.0);

		for &DumbQuad {region: Region{context, pos, size}, color} in 
			self.dumb_quads.iter().filter(|q| q.region.context.is_screen())
		{
			let size = size / 2.0;
			let color = color.into();

			let origin = match context {
				ScreenCenter => Vec3::zero(),
				ScreenTopLeft => Vec3::new(-1.0,  1.0, 0.0) * aspect,
				ScreenBottomLeft => Vec3::new(-1.0, -1.0, 0.0) * aspect,
				ScreenTopRight => Vec3::new( 1.0,  1.0, 0.0) * aspect,
				ScreenBottomRight => Vec3::new( 1.0, -1.0, 0.0) * aspect,

				_ => unreachable!()
			};

			mb.add_quad(&[
				UiVertex::new(Vec3::new(-size.x, -size.y, 0.0) + pos + origin, color),
				UiVertex::new(Vec3::new(-size.x,  size.y, 0.0) + pos + origin, color),
				UiVertex::new(Vec3::new( size.x,  size.y, 0.0) + pos + origin, color),
				UiVertex::new(Vec3::new( size.x, -size.y, 0.0) + pos + origin, color),
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


	pub fn update_immediate_interact_region(&mut self, hoverable: &mut Hoverable, region: &Region) -> bool {
		hoverable.update();

		if region.includes(self) {
			hoverable.begin_hover();

			if self.click_occured {
				hoverable.click();
				// Don't click more than one thing
				self.click_occured = false;

				return true;
			}

		} else {
			hoverable.end_hover();
		}

		false
	}


	pub fn quad<R>(&mut self, region: R, color: Color)
		where R: Into<Region>
	{
		let region = region.into();
		self.dumb_quads.push(DumbQuad { region, color });
	}

	pub fn arrow<R>(&mut self, region: R, direction: Direction, color: Color)
		where R: Into<Region>
	{
		let region = region.into();
		self.dumb_arrows.push(DumbArrow { region, direction, color });
	}
}



struct DumbQuad {
	region: Region,
	color: Color,
}

struct DumbArrow {
	region: Region,
	direction: Direction,
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
	pub fn new(pos: Vec3, size: Vec2, context: Context) -> Region {
		Region {
			pos, size,
			context,
		}
	}

	pub fn new_ground(pos: Vec3, size: Vec2) -> Region {
		Region {
			pos, size,
			context: Context::Ground,
		}
	}

	fn includes(&self, ui: &Ui) -> bool {
		let (self_pos, mouse_pos) = match self.context {
			ctx if ctx.is_screen() => {
				let aspect = aspect_to_vec(ui.aspect);
				let view_mouse = ui.screen_mouse * aspect;

				let origin = match ctx {
					Context::ScreenCenter => Vec2::zero(),
					Context::ScreenTopLeft => Vec2::new(-1.0,  1.0) * aspect,
					Context::ScreenBottomLeft => Vec2::new(-1.0, -1.0) * aspect,
					Context::ScreenTopRight => Vec2::new( 1.0,  1.0) * aspect,
					Context::ScreenBottomRight => Vec2::new( 1.0, -1.0) * aspect,

					_ => unreachable!()
				};

				(self.pos.to_xy() + origin, view_mouse)
			}

			ctx if ctx.is_world() => {
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

			_ => unreachable!()
		};

		let diff = self_pos - mouse_pos;
		let extent = self.size / 2.0;

		diff.x.abs() < extent.x && diff.y.abs() < extent.y
	}
}


impl From<(Vec3, Vec2, Context)> for Region {
	fn from((pos, size, context): (Vec3, Vec2, Context)) -> Region {
		Region::new(pos, size, context)
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



pub struct HoverablePalette {
	pub base: Color,
	pub hover: Color,
	pub click: Color,
}

impl HoverablePalette {
	pub fn new(base: Color) -> HoverablePalette {
		HoverablePalette {
			base,
			hover: 0.5f32.ease_linear(base, Color::white()),
			click: Color::white(),
		}
	}

	pub fn with_hover(self, hover: Color) -> HoverablePalette {
		HoverablePalette { hover, .. self }
	}

	pub fn color(&self, state: HoverState) -> Color {
		use HoverState::*;

		match state {
			Idle => self.base,
			HoverEnter(v) | HoverExit(v) => v.ease_linear(self.base, self.hover),
			Hovering => self.hover,
			Clicked(v) => v.ease_back_in(self.hover, self.click),
		}
	}
}


fn aspect_to_vec(aspect: f32) -> Vec2 {
	if aspect < 1.0 {
		Vec2::new(1.0, 1.0 / aspect)
	} else {
		Vec2::new(aspect, 1.0)
	}
}





pub struct GlobalPalette {
	pub health: HoverablePalette,
	pub food: HoverablePalette,
	pub hunger: HoverablePalette,
	pub map: HoverablePalette,
	pub treasure: HoverablePalette,
	pub movement: HoverablePalette,
}

static mut GLOBAL_PALETTE: Option<GlobalPalette> = None;

pub fn palette() -> &'static GlobalPalette {
	unsafe {
		GLOBAL_PALETTE.get_or_insert_with(|| {
			GlobalPalette {
				health: HoverablePalette::new(Color::rgb(0.9, 0.3, 0.3)),
				hunger: HoverablePalette::new(Color::rgb(0.8, 0.7, 0.3)),
				map: HoverablePalette::new(Color::rgb(0.5, 0.0, 1.0)),
				food: HoverablePalette::new(Color::rgb(0.6, 0.5, 0.2)),
				treasure: HoverablePalette::new(Color::rgb(0.3, 0.6, 0.2)),
				movement: HoverablePalette::new(Color::rgba(1.0, 0.4, 0.5, 0.2))
					.with_hover(Color::rgb(1.0, 0.4, 0.5)),
			}
		})
	}
}