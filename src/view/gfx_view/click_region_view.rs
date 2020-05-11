use crate::prelude::*;
use crate::task::{PlayerCommand, ControllerMode};
use super::gfx::{Gfx, MeshID};
use super::mesh_builder::MeshBuilder;
use super::vertex::ColorVertex;

type ColorMeshBuilder = MeshBuilder<ColorVertex>;

pub struct ClickRegionView {
	mb: ColorMeshBuilder,
	regions: Vec<ClickRegion<PlayerCommand>>,
}


impl ClickRegionView {
	pub fn new(gfx: &mut Gfx) -> Self {
		Self {
			mb: ColorMeshBuilder::new(gfx.new_mesh()),
			regions: Vec::new(),
		}
	}

	pub fn render(&mut self, gfx: &mut Gfx) {
		self.mb.clear();

		for region in self.regions.iter_mut() { region.update() }

		let base_col = Color::rgba(1.0, 0.4, 0.5, 0.4);
		let hover_col = Color::rgba(1.0, 0.4, 0.5, 1.0);
		let click_col = Color::white();

		for region in self.regions.iter() {
			use ClickRegionState::*;

			let (size, color) = match region.state {
				Idle => (0.0, base_col),
				Hovering(v) | HoverExit(v) => (v*0.02, v.ease_linear(base_col, hover_col)),
				Clicked(v) => (0.02, v.ease_back_in(hover_col, click_col)),
			};

			build_square(&mut self.mb, region.pos, region.size + size, color);
		}

		gfx.update_mesh_from(&self.mb);
		gfx.draw_mesh(self.mb.mesh_id);
	}

	pub fn gen_regions_for_room(&mut self, pos: Vec2) {
		use crate::controller::main::PlayerCommand as MainCmd;
		
		self.regions = vec![
			ClickRegion {
				pos: pos + Vec2::new(1.0, 0.0),
				size: 0.3,
				state: ClickRegionState::Idle,
				cmd: || PlayerCommand::Main(MainCmd::GoEast)
			},

			ClickRegion {
				pos: pos + Vec2::new(-1.0, 0.0),
				size: 0.3,
				state: ClickRegionState::Idle,
				cmd: || PlayerCommand::Main(MainCmd::GoWest)
			},

			ClickRegion {
				pos: pos + Vec2::new(0.0, 1.0),
				size: 0.3,
				state: ClickRegionState::Idle,
				cmd: || PlayerCommand::Main(MainCmd::GoSouth)
			},

			ClickRegion {
				pos: pos + Vec2::new(0.0, -1.0),
				size: 0.3,
				state: ClickRegionState::Idle,
				cmd: || PlayerCommand::Main(MainCmd::GoNorth)
			},
		];
	}

	pub fn process_click(&mut self, world: Vec2, ctl_mode: ControllerMode) -> Option<PlayerCommand> {
		if ctl_mode != ControllerMode::Main { return None }

		for r in self.regions.iter_mut() {
			if r.contains(world) {
				return Some(r.process_click());
			}
		}

		None
	}

	pub fn process_hover(&mut self, world: Vec2) {
		for r in self.regions.iter_mut() {
			if r.contains(world) {
				r.process_hover();
			} else {
				r.process_no_hover();
			}
		}
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



enum ClickRegionState {
	Idle,
	Hovering(f32),
	HoverExit(f32),
	Clicked(f32),
}


struct ClickRegion<Cmd> {
	pos: Vec2,
	size: f32,
	state: ClickRegionState,

	cmd: fn() -> Cmd,
}

impl<Cmd> ClickRegion<Cmd> {
	fn contains(&self, point: Vec2) -> bool {
		let Vec2{x, y} = point - self.pos;
		let extent = self.size / 2.0;

		x.abs() < extent && y.abs() < extent 
	}

	fn update(&mut self) {
		use ClickRegionState::*;

		let hover_fade_rate = 0.1;
		let click_fade_rate = 0.1;

		self.state = match self.state {
			Idle => Idle,

			Hovering(v) => Hovering((v+hover_fade_rate).min(1.0)),

			HoverExit(v) => if v > 0.0 {
				HoverExit((v-hover_fade_rate).max(0.0))
			} else {
				Idle
			},

			Clicked(v) => if v > 0.0 { Clicked((v-click_fade_rate).max(0.0)) } else { Hovering(1.0) },
		};
	}

	fn process_hover(&mut self) {
		use ClickRegionState::*;

		self.state = match self.state {
			Idle => Hovering(0.0),
			HoverExit(v) => Hovering(v),
			_ => return,
		};
	}

	fn process_no_hover(&mut self) {
		use ClickRegionState::*;

		self.state = match self.state {
			Hovering(v) => HoverExit(v),
			_ => return,
		};
	}

	fn process_click(&mut self) -> Cmd {
		self.state = ClickRegionState::Clicked(1.0);
		(self.cmd)()
	}
}