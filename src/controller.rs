use crate::prelude::*;
use crate::game_state::GameState;
use crate::task::Executor;

pub mod main;
pub mod battle;
pub mod merchant;
pub use main::run_main_controller;
pub use battle::run_battle_controller;
pub use merchant::run_merchant_controller;

pub trait ControllerTrait: std::fmt::Debug {
	fn enter(&mut self, _ctx: &mut ControllerContext) {}
	fn leave(&mut self, _ctx: &mut ControllerContext) {}
	fn run_command(&mut self, ctx: &mut ControllerContext, command: &str) -> Option<Event>;
}

pub enum Event {
	// TransitionTo(Controller),
	// Enter(Controller),
	Leave,
	
	Win,
	Lose,

	Restart,
	Quit,
}

pub struct ControllerContext<'gs> {
	pub state: &'gs mut GameState,
}

impl<'gs> ControllerContext<'gs> {
	pub fn new(state: &'gs mut GameState) -> Self {
		ControllerContext { state }
	}
}



// #[derive(Debug)]
// pub enum Controller {
// 	Main(MainController),
// 	Battle(BattleController),
// 	Merchant(MerchantController),
// }

// impl Controller {
// 	pub fn enter(&mut self, ctx: &mut ControllerContext) {
// 		match self {
// 			Controller::Main(ctl) => ctl.enter(ctx),
// 			Controller::Battle(ctl) => ctl.enter(ctx),
// 			Controller::Merchant(ctl) => ctl.enter(ctx),
// 		}
// 	}

// 	pub fn leave(&mut self, ctx: &mut ControllerContext) {
// 		match self {
// 			Controller::Main(ctl) => ctl.leave(ctx),
// 			Controller::Battle(ctl) => ctl.leave(ctx),
// 			Controller::Merchant(ctl) => ctl.leave(ctx),
// 		}
// 	}

// 	pub fn run_command(&mut self, ctx: &mut ControllerContext, command: &str) -> Option<Event> {
// 		match self {
// 			Controller::Main(ctl) => ctl.run_command(ctx, command),
// 			Controller::Battle(ctl) => ctl.run_command(ctx, command),
// 			Controller::Merchant(ctl) => ctl.run_command(ctx, command),
// 		}
// 	}
// }

// impl From<MainController> for Controller {
// 	fn from(ctl: MainController) -> Controller {
// 		Controller::Main(ctl)
// 	}
// }

// impl From<BattleController> for Controller {
// 	fn from(ctl: BattleController) -> Controller {
// 		Controller::Battle(ctl)
// 	}
// }

// impl From<MerchantController> for Controller {
// 	fn from(ctl: MerchantController) -> Controller {
// 		Controller::Merchant(ctl)
// 	}
// }