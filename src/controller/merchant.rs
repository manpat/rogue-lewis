// use crate::prelude::*;
use crate::task;
use crate::game_state::Item;


pub async fn run_merchant_controller() {
	println!("[merchant] enter");

	println!("The merchant greets you");

	loop {
		match task::get_player_command().await.0.as_str() {
			"food" => if task::consume_player_item(Item::Treasure).await {
				task::give_player_item(Item::Food).await
			} else {
				println!("You don't have enough treasure!");
			}

			"map" => if task::consume_player_item_n(Item::Treasure, 3).await {
				task::give_player_item(Item::Map).await
			} else {
				println!("You don't have enough treasure!");
			}

			cmd => {
				println!("The merchant is insulted by whatever it is you just said and forces you to leave");
				println!("player input {:?}", cmd);
				break
			}
		}
	}

	println!("[merchant] leave");
}