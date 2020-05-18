use crate::prelude::*;
use super::util::*;
use super::gfx::{Gfx, ui};

use crate::task::ControllerMode;
use crate::gamestate::GameState;
use crate::controller::battle::PlayerCommand::*;


pub struct BattleView {
	active: bool,

	flee_button: ui::Hoverable,
	attack_button: ui::Hoverable,
	heal_button: ui::Hoverable,
}


impl BattleView {
	pub fn new() -> Self {
		BattleView {
			active: false,

			flee_button: Default::default(),
			attack_button: Default::default(),
			heal_button: Default::default(),
		}
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Battle);
		if !self.active {
			self.flee_button.reset();
		}
	}

	pub fn update(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let size = Vec2::splat(0.2);
		let room_pos = location_to_world(gamestate.player.location).to_x0z();

		let flee_palette = ui::HoverablePalette::new(Color::rgb(0.5, 0.0, 1.0));
		let attack_palette = ui::HoverablePalette::new(Color::rgb(1.0, 0.0, 0.0));
		let heal_palette = ui::HoverablePalette::new(Color::rgb(0.0, 0.8, 0.8));

		let flee_button_pos = room_pos + Vec3::new(0.7, 0.01, -0.5);
		let attack_button_pos = room_pos + Vec3::new(0.7, 0.01, -0.2);
		let heal_button_pos = room_pos + Vec3::new(0.7, 0.01, 0.1);

		// Flee button
		gfx.ui().update_interact_region(
			&mut self.flee_button,
			&ui::Region::new_ground(flee_button_pos, size),
			|| Flee
		);

		let color = flee_palette.color(self.flee_button.state());
		gfx.ui().arrow(Direction::East, flee_button_pos, 0.2, color, ui::Context::Ground);


		// Attack button
		gfx.ui().update_interact_region(
			&mut self.attack_button,
			&ui::Region::new_ground(attack_button_pos, size),
			|| Attack
		);

		let color = attack_palette.color(self.attack_button.state());
		gfx.ui().quad(attack_button_pos, size, color, ui::Context::Ground);


		// Heal button
		gfx.ui().update_interact_region(
			&mut self.heal_button,
			&ui::Region::new_ground(heal_button_pos, size),
			|| Heal
		);

		let color = heal_palette.color(self.heal_button.state());
		gfx.ui().quad(heal_button_pos, size, color, ui::Context::Ground);
	}
}