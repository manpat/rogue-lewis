pub mod text_view;
pub mod gfx_view;

pub use text_view::TextView;
pub use gfx_view::GfxView;

use crate::gamestate::{GameState, GameCommand};
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
	fn update(&mut self, gamestate: &GameState);
	fn should_quit(&self) -> bool;
}
