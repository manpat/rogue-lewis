use crate::prelude::*;
use crate::controller::*;

#[derive(Debug)]
pub struct MerchantController {
	// state: main menu/sell/buy
	// merchant
}

impl ControllerTrait for MerchantController {
	fn enter(&mut self, _: &mut ControllerContext<'_>) {
		println!("The merchant greets you");
	}

	fn run_command(&mut self, _: &mut ControllerContext<'_>, command: &str) -> Option<Event> {
		match command {
			_ => {
				println!("The merchant is insulted by whatever it is you just said and forces you to leave");
				Some(Event::Leave)
			}
		}

	}
}