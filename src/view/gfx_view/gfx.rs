pub mod core;
pub mod ui;
pub mod mesh;
pub mod shader;
pub mod vertex;
pub mod mesh_builder;
pub mod camera;

use crate::prelude::*;
use mesh_builder::MeshBuilder;

pub struct Gfx {
	pub core: core::Core,
	pub camera: camera::Camera,
	pub ui: ui::Ui,

	ui_builder: MeshBuilder<ui::UiVertex>,
}


impl Gfx {
	pub fn new() -> Gfx {
		let mut core = core::Core::new();

		let ui_mesh_id = core.new_mesh();
		let ui_builder = MeshBuilder::new(ui_mesh_id);

		unsafe {
			gl::Enable(gl::DEPTH_TEST);
		}

		Gfx {
			core,
			camera: camera::Camera::new(),

			ui: ui::Ui::new(),
			ui_builder,
		}
	}

	pub fn draw_world_ui(&mut self) {
		self.ui_builder.clear();
		self.ui.build_world_space(&mut self.ui_builder);
		self.core.update_mesh_from(&self.ui_builder);
		self.core.draw_mesh(self.ui_builder.mesh_id);
	}

	pub fn draw_screen_ui(&mut self) {
		self.ui_builder.clear();
		self.ui.build_screen_space(&mut self.ui_builder);
		self.core.update_mesh_from(&self.ui_builder);
		self.core.draw_mesh(self.ui_builder.mesh_id);
	}
}