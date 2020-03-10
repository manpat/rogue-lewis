use crate::prelude::*;
use crate::game_state::GameState;

pub mod main;
pub mod battle;
pub mod merchant;
pub use main::MainController;
pub use battle::BattleController;
pub use merchant::MerchantController;

pub trait Controller: std::fmt::Debug {
	fn init(&mut self, _state: &GameState) {}
	fn run_command(&mut self, state: &mut GameState, command: &str) -> Option<Event>;
}

pub enum Event {
	Transition(Box<dyn Controller>),
	
	Win,
	Lose,

	Restart,
	Quit,
}