pub mod executor;
pub mod promise;
pub mod coordinator;

pub use promise::{UntypedPromise, Promise, Promisable, FutureValue};
pub use executor::Executor;
pub use coordinator::Coordinator;

use crate::prelude::*;

use crate::view::ViewCommand;
use crate::game_state::GameCommand;

#[derive(Copy, Clone, Debug)]
pub enum ControllerMode {
	Main, Battle, Merchant
}

use crate::controller::{main, battle, merchant};

#[derive(Debug)]
pub enum PlayerCommand {
	Main(main::PlayerCommand),
	Battle(battle::PlayerCommand),
	Merchant(merchant::PlayerCommand),
	Debug(Vec<String>),
}

impl PlayerCommand {
	pub fn main(&self) -> Option<&main::PlayerCommand> {
		match self {
			PlayerCommand::Main(cmd) => Some(cmd),
			_ => None,
		}
	}

	pub fn battle(&self) -> Option<&battle::PlayerCommand> {
		match self {
			PlayerCommand::Battle(cmd) => Some(cmd),
			_ => None,
		}
	}

	pub fn merchant(&self) -> Option<&merchant::PlayerCommand> {
		match self {
			PlayerCommand::Merchant(cmd) => Some(cmd),
			_ => None,
		}
	}

	pub fn debug(&self) -> Option<&[String]> {
		match self {
			PlayerCommand::Debug(cmd) => Some(cmd),
			_ => None,
		}
	}
}

pub async fn enter_mode(mode: ControllerMode) {
	get_coordinator()
		.schedule_view_command(ViewCommand::PushControllerMode(mode))
		.await
}

pub async fn leave_mode() {
	get_coordinator()
		.schedule_view_command(ViewCommand::PopControllerMode)
		.await
}


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

use crate::game_state::HealthModifyReason;
use crate::item::Item;

// TODO: consume/interact_room_encounter/item?

pub async fn give_player_item(item: Item) {
	give_player_item_n(item, 1).await
}

pub async fn give_player_item_n(item: Item, n: usize) {
	let command = GameCommand::GivePlayerItem(item, n);
	get_coordinator().schedule_model_command::<()>(command).await;
	get_coordinator().schedule_view_command(ViewCommand::GameCommand(command)).await
}

pub async fn consume_player_item(item: Item) -> bool {
	consume_player_item_n(item, 1).await
}

pub async fn consume_player_item_n(item: Item, n: usize) -> bool {
	let command = GameCommand::ConsumePlayerItem(item, n);

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

pub async fn heal_player(n: u32) {
	let command = GameCommand::ModifyPlayerHealth(n as i32, HealthModifyReason::Heal);
	get_coordinator().schedule_model_command::<bool>(command).await;
	get_coordinator().schedule_view_command(ViewCommand::GameCommand(command)).await
}

pub async fn damage_player(n: u32, reason: HealthModifyReason) -> bool {
	let command = GameCommand::ModifyPlayerHealth(-(n as i32), reason);
	let still_alive = get_coordinator().schedule_model_command(command).await;
	// TODO: pass status to view
	get_coordinator().schedule_view_command::<()>(ViewCommand::GameCommand(command)).await;
	still_alive
}

pub async fn move_player(dir: Direction) -> bool {
	let command = GameCommand::MovePlayer(dir);
	let did_move = get_coordinator().schedule_model_command(command).await;
	if did_move {
		// TODO: pass success to view
		get_coordinator().schedule_view_command::<()>(ViewCommand::GameCommand(command)).await;
	}
	did_move
}