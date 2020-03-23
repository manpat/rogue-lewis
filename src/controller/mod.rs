use crate::prelude::*;
use crate::game_state::GameState;

pub mod main;
pub mod battle;
pub mod merchant;
pub use main::MainController;
pub use battle::BattleController;
pub use merchant::MerchantController;

pub trait Controller: std::fmt::Debug {
	fn enter(&mut self, _state: &mut GameState) {}
	fn leave(&mut self, _state: &mut GameState) {}
	fn run_command(&mut self, state: &mut GameState, command: &str) -> Option<Event>;
}

pub enum Event {
	TransitionTo(Box<dyn Controller>),
	Enter(Box<dyn Controller>),
	Leave,
	
	Win,
	Lose,

	Restart,
	Quit,
}