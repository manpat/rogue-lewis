use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui};

use crate::task::ControllerMode;
use crate::gamestate::GameState;
use crate::controller::battle::PlayerCommand::*;


pub struct BattleView {
	active: bool,

	flee_button: ui::Hoverable,
}


impl BattleView {
	pub fn new() -> Self {
		BattleView {
			active: false,

			flee_button: Default::default(),
		}
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Battle);
		if !self.active {
			self.flee_button.reset();
		}
	}

	pub fn render(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let size = Vec2::splat(0.2);
		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, -0.5);

		gfx.ui().update_interact_region(
			&mut self.flee_button,
			&ui::Region::new_ground(pos, size),
			|| Flee
		);

		let color = match self.flee_button.state() {
			ui::HoverState::Clicked(_) => Color::rgb(1.0, 1.0, 1.0),
			ui::HoverState::Hovering
			| ui::HoverState::HoverEnter(_)
			| ui::HoverState::HoverExit(_) => Color::rgb(1.0, 0.5, 0.5),

			_ => Color::rgb(1.0, 0.0, 0.0),
		};

		gfx.ui().quad(pos, size, color, ui::Context::Ground);

		let pos = location_to_world(gamestate.player.location).to_x0z() + Vec3::new(0.7, 0.01, -0.2);
		let color = Color::rgb(0.0, 1.0, 0.0);

		gfx.ui().quad(pos, size, color, ui::Context::Ground);
	}
}