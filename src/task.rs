pub mod executor;
pub mod promise;
pub mod coordinator;

pub use promise::{UntypedPromise, Promise, Promisable, FutureValue};
pub use executor::Executor;
pub use coordinator::Coordinator;

use crate::prelude::*;
use coordinator::*;
use crate::view::ViewCommand;

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
		.schedule_view_command(ViewCommand::GetPlayerCommand)
		.await
}

pub async fn show_map(whole_map: bool) {
	get_coordinator()
		.schedule_view_command(ViewCommand::ShowMap {whole_map})
		.await
}

use crate::game_state::Item;

#[derive(Copy, Clone, Debug)]
pub enum GameCommand {
	GivePlayerItem(crate::game_state::Item),
	ConsumePlayerItem(crate::game_state::Item),
}

pub async fn give_player_item(item: Item) {
	let command = GameCommand::GivePlayerItem(item);
	get_coordinator().schedule_model_command::<()>(command).await;
	get_coordinator().schedule_view_command(ViewCommand::GameCommand(command)).await
}

// TODO: consume/interact_room_encounter/item?

pub async fn consume_player_item(item: Item) -> bool {
	let command = GameCommand::ConsumePlayerItem(item);

	let success = get_coordinator().schedule_model_command(command).await;
	if success {
		// TODO: maybe I want an event on failure and success?
		// or maybe I want something more specific than just forwarding the GameCommand
		get_coordinator()
			.schedule_view_command(ViewCommand::GameCommand(command))
			.await
	}

	success
}
