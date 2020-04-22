// use crate::prelude::*;
use crate::task;


pub async fn run_merchant_controller() {
	println!("[merchant] enter");

	println!("The merchant greets you");

	match task::get_player_command().await {
		cmd => {
			println!("The merchant is insulted by whatever it is you just said and forces you to leave");
			println!("player input {:?}", cmd);
		}
	}

	println!("[merchant] leave");
}