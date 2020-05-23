use crate::prelude::*;
use super::gfx::{Gfx, ui};

use crate::gamestate::GameState;
use crate::item::Item;

pub struct HudView {
	map_hoverable: ui::Hoverable,
}

impl HudView {
	pub fn new() -> Self {
		HudView {
			map_hoverable: Default::default(),
		}
	}

	pub fn update(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		let margin = 0.01;
		let width = 0.03;
		let height = 0.06;

		for i in 0..gamestate.player.health {
			let pos_x = i as f32 * (width + margin) + (width/2.0 + margin);
			let pos_y = height/2.0 + margin;

			let pos = Vec3::new(pos_x, pos_y, 0.0);
			let color = ui::palette().health.base;

			gfx.ui.quad(pos, Vec2::new(width, height), color, ui::Context::ScreenBottomLeft);
		}

		for i in 0..gamestate.player.hunger {
			let pos_x = i as f32 * (width + margin) + (width/2.0 + margin);
			let pos_y = height/2.0 + margin + (height+margin);

			let pos = Vec3::new(pos_x, pos_y, 0.0);
			let color = ui::palette().hunger.base;

			gfx.ui.quad(pos, Vec2::new(width, height), color, ui::Context::ScreenBottomLeft);
		}

		for i in 0..gamestate.player.inventory.count(Item::Treasure) {
			let pos_x = -(i as f32 * (width + margin) + (width/2.0 + margin));
			let pos_y = height/2.0 + margin;

			let pos = Vec3::new(pos_x, pos_y, 0.0);
			let color = ui::palette().treasure.base;

			gfx.ui.quad(pos, Vec2::new(width, height), color, ui::Context::ScreenBottomRight);
		}

		for i in 0..gamestate.player.inventory.count(Item::Food) {
			let pos_x = -(i as f32 * (width + margin) + (width/2.0 + margin));
			let pos_y = height/2.0 + margin + (height+margin);

			let pos = Vec3::new(pos_x, pos_y, 0.0);
			let color = ui::palette().food.base;

			gfx.ui.quad(pos, Vec2::new(width, height), color, ui::Context::ScreenBottomRight);
		}

		// TODO: hide in battle/merchant modes
		if gamestate.player.inventory.has(Item::Map) {
			let size = Vec2::splat(0.2);
			let pos = Vec3::new(0.11, -0.11, 0.0);

			let region = ui::Region::new(pos, size, ui::Context::ScreenTopLeft);

			gfx.ui.update_interact_region(
				&mut self.map_hoverable,
				&region,
				|| crate::controller::main::PlayerCommand::ShowMap
			);

			let color = ui::palette().map.color(self.map_hoverable.state());
			gfx.ui.quad(pos, size, color, ui::Context::ScreenTopLeft);

		} else {
			self.map_hoverable.reset();
		}
	}
}