use crate::prelude::*;
use crate::task::ControllerMode;
use crate::gamestate::GameState;
use crate::controller::merchant::PlayerCommand::*;

use super::gfx::{Gfx, ui};
use super::util::*;


pub struct MerchantView {
	active: bool,

	leave_button: ui::Hoverable,
}


impl MerchantView {
	pub fn new() -> Self {
		Self {
			active: false,

			leave_button: Default::default(),
		}
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Merchant);
		if !self.active {
			self.leave_button.reset();
		}
	}

	pub fn render(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let size = Vec2::splat(0.2);
		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, 0.5);

		gfx.ui().update_interact_region(
			&mut self.leave_button,
			&ui::Region::new_ground(pos, size),
			|| Leave
		);


		let color = match self.leave_button.state() {
			ui::HoverState::Clicked(_) => Color::rgb(1.0, 1.0, 1.0),
			ui::HoverState::Hovering
			| ui::HoverState::HoverEnter(_) => Color::rgb(1.0, 0.5, 1.0),

			_ => Color::rgb(1.0, 0.0, 1.0),
		};

		gfx.ui().quad(pos, size, color, ui::Context::Ground);
	}
}