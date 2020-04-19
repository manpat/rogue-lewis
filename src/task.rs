pub mod executor;
pub mod coordinator;

pub use executor::Executor;
pub use coordinator::Coordinator;

use crate::prelude::*;
use coordinator::*;


pub async fn get_player_command() -> String {
	get_coordinator().schedule_value_await(WakeEvent::GetPlayerCommand, |inner| inner.player_command.take()).await
}

pub async fn show_map(whole_map: bool) {
	get_coordinator().schedule_event_await(WakeEvent::ShowMap {whole_map}).await
}