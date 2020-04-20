pub mod executor;
pub mod coordinator;

pub use executor::Executor;
pub use coordinator::Coordinator;

use crate::prelude::*;
use coordinator::*;

// pub enum ControllerMode {
// 	Battle, Merchant
// }

// pub enum ControllerModeChange {
// 	Enter(ControllerMode),
// 	Leave(ControllerMode),
// }

#[derive(Debug)]
pub struct PlayerCommand(pub String);

// pub async fn enter_mode(mode: ControllerMode) {
// 	get_coordinator()
// 		.schedule_mode_change(ControllerModeChange::Enter(mode))
// 		.await
// }

// pub async fn leave_mode(mode: ControllerMode) {
// 	get_coordinator()
// 		.schedule_mode_change(ControllerModeChange::Leave(mode))
// 		.await
// }

pub async fn get_player_command() -> PlayerCommand {
	get_coordinator()
		.schedule_value_await(WakeEvent::GetPlayerCommand, |inner| inner.player_command.take())
		.await
}

pub async fn show_map(whole_map: bool) {
	get_coordinator()
		.schedule_event_await(WakeEvent::ShowMap {whole_map})
		.await
}

use crate::game_state::Item;

#[derive(Copy, Clone, Debug)]
pub enum ControllerEvent {
	PlayerGotItem(crate::game_state::Item),
	// PlayerConsumeItem(crate::game_state::Item),
}

pub async fn give_player_item(item: Item) {
	get_coordinator()
		.notify_controller_event(ControllerEvent::PlayerGotItem(item))
		.await
}

// TODO: how does failure information get back to the future?
// and how does it do so before the view responds?
// pub async fn consume_player_item(item: Item) /*-> bool*/ {
// 	get_coordinator()
// 		.notify_controller_event(ControllerEvent::PlayerConsumeItem(item))
// 		.await
// }
