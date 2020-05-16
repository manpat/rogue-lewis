use crate::prelude::*;
use crate::task::{PlayerCommand, ControllerMode};
use crate::gamestate::GameState;

use super::gfx::{Gfx, ui::Context as UiContext};
use super::click_region::*;
use super::util::*;


pub struct MerchantView {
	active: bool,
}


impl MerchantView {
	pub fn new() -> Self {
		Self {
			active: false,
		}
	}

	pub fn process_mouse_event(&mut self, event: ClickRegionEvent) -> Option<PlayerCommand> {
		use crate::controller::merchant::PlayerCommand::*;
		Some(Leave.into())
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Merchant);
	}

	pub fn render(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, 0.5);
		let color = Color::rgb(1.0, 0.0, 1.0);

		gfx.ui().quad(pos, Vec2::splat(0.2), color, UiContext::World);
	}
}