use crate::prelude::*;
use crate::task::Coordinator;


pub async fn run_merchant_controller(ctx: Coordinator) {
	println!("[merchant] enter");

	println!("The merchant greets you");

	match ctx.get_player_command().await {
		cmd => {
			println!("The merchant is insulted by whatever it is you just said and forces you to leave");
			println!("player input {}", cmd);
		}
	}

	println!("[merchant] leave");
}