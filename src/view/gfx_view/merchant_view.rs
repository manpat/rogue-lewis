use crate::prelude::*;
use crate::task::ControllerMode;
use crate::gamestate::GameState;
use crate::item::Item;
use crate::controller::merchant::PlayerCommand::*;

use super::gfx::{Gfx, ui};
use super::util::*;


pub struct MerchantView {
	active: bool,

	buy_map_button: ui::Hoverable,
	buy_food_button: ui::Hoverable,
	buy_key_button: ui::Hoverable,
	buy_potion_button: ui::Hoverable,

	leave_button: ui::Hoverable,
}


impl MerchantView {
	pub fn new() -> Self {
		Self {
			active: false,

			buy_map_button: Default::default(),
			buy_food_button: Default::default(),
			buy_key_button: Default::default(),
			buy_potion_button: Default::default(),

			leave_button: Default::default(),
		}
	}

	pub fn on_mode_change(&mut self, mode: ControllerMode) {
		self.active = matches!(mode, ControllerMode::Merchant);
		if !self.active {
			self.leave_button.reset();
		}
	}

	pub fn update(&mut self, gfx: &mut Gfx, gamestate: &GameState) {
		if !self.active { return }

		let size = Vec2::splat(0.2);
		let map_pos = location_to_world(gamestate.player.location).to_x0z();

		let leave_pos = map_pos + Vec3::new(0.7, 0.01, -0.5);

		let buy_map_pos = map_pos + Vec3::new(-0.4, 0.01, 0.7);
		let buy_food_pos = map_pos + Vec3::new(-0.1, 0.01, 0.7);
		let buy_key_pos = map_pos + Vec3::new( 0.1, 0.01, 0.7);
		let buy_potion_pos = map_pos + Vec3::new( 0.4, 0.01, 0.7);

		gfx.ui.update_interact_region(
			&mut self.buy_map_button,
			&ui::Region::new_ground(buy_map_pos, size),
			|| BuyItem(Item::Map)
		);

		gfx.ui.update_interact_region(
			&mut self.buy_food_button,
			&ui::Region::new_ground(buy_food_pos, size),
			|| BuyItem(Item::Food)
		);

		gfx.ui.update_interact_region(
			&mut self.buy_key_button,
			&ui::Region::new_ground(buy_key_pos, size),
			|| BuyItem(Item::Key)
		);

		gfx.ui.update_interact_region(
			&mut self.buy_potion_button,
			&ui::Region::new_ground(buy_potion_pos, size),
			|| BuyItem(Item::Potion)
		);

		gfx.ui.update_interact_region(
			&mut self.leave_button,
			&ui::Region::new_ground(leave_pos, size),
			|| Leave
		);

		let color = ui::palette().map.color(self.buy_map_button.state());
		gfx.ui.quad(buy_map_pos, size, color, ui::Context::Ground);

		let color = ui::palette().map.color(self.buy_food_button.state());
		gfx.ui.quad(buy_food_pos, size, color, ui::Context::Ground);

		let color = ui::palette().map.color(self.buy_key_button.state());
		gfx.ui.quad(buy_key_pos, size, color, ui::Context::Ground);

		let color = ui::palette().map.color(self.buy_potion_button.state());
		gfx.ui.quad(buy_potion_pos, size, color, ui::Context::Ground);

		let color = ui::palette().map.color(self.leave_button.state());
		gfx.ui.arrow(Direction::East, leave_pos, 0.2, color, ui::Context::Ground);
	}
}