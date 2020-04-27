pub mod text_view;

pub use text_view::TextView;

use crate::game_state::{GameState, GameCommand};
use crate::task::{UntypedPromise, ControllerMode};

#[derive(Copy, Clone)]
pub enum ViewCommand {
	GetPlayerCommand,
	ShowMap { whole_map: bool },
	ShowInventory,
	GameCommand(GameCommand),
	PushControllerMode(ControllerMode),
	PopControllerMode,
}


pub trait View {
	fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise);
	fn update(&mut self, game_state: &GameState);
}
