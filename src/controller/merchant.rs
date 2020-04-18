use crate::prelude::*;
use crate::controller::*;
use crate::task;

// #[derive(Debug)]
// pub struct MerchantController {
// 	// state: main menu/sell/buy
// 	// merchant
// }

// impl ControllerTrait for MerchantController {
// 	fn enter(&mut self, ctx: &mut ControllerContext) {
// 		// ctx.executor.queue(run_merchant_controller());

// 		// println!("The merchant greets you");
// 	}

// 	fn run_command(&mut self, _: &mut ControllerContext, _: &str) -> Option<Event> {
// 		// match command {
// 		// 	_ => {
// 		// 		println!("The merchant is insulted by whatever it is you just said and forces you to leave");
// 		// 		Some(Event::Leave)
// 		// 	}
// 		// }
// 		None
// 	}
// }


pub async fn run_merchant_controller() {
	println!("[merchant] enter");

	println!("The merchant greets you");

	match task::get_player_command().await {
		cmd => {
			println!("The merchant is insulted by whatever it is you just said and forces you to leave");
			println!("player input {}", cmd);
		}
	}

	println!("[merchant] leave");
}