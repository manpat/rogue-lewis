use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui::Context as UiContext};
use super::click_region::*;

use crate::task::{PlayerCommand, ControllerMode};
use crate::gamestate::GameState;


pub struct BattleView {
	active: bool,
}


impl BattleView {
	pub fn new() -> Self {
		BattleView {
			active: false,
		}
	}

	pub fn process_mouse_event(&mut self, event: ClickRegionEvent) -> Option<PlayerCommand> {
		use crate::controller::battle::PlayerCommand::*;

		Some(Flee.into())
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Battle);
	}

	pub fn render(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let size = 0.2;
		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, -0.5);
		let color = Color::rgb(1.0, 0.0, 0.0);

		gfx.ui().quad(pos, Vec2::splat(0.2), color, UiContext::World);

		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, -0.2);
		let color = Color::rgb(0.0, 1.0, 0.0);

		gfx.ui().quad(pos, Vec2::splat(0.2), color, UiContext::World);
	}
}