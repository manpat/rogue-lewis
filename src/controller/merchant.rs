// use crate::prelude::*;
use crate::task;
use crate::game_state::Item;


#[derive(Debug)]
pub enum PlayerCommand {
	BuyItem(Item),
	SellItem(Item),

	Leave,
}


fn item_cost(item: Item) -> usize {
	match item {
		Item::Food => 1,
		Item::Map => 3,
		Item::Key => 5,

		Item::Treasure => panic!("Treasure has no cost"),
	}
}


pub async fn run_merchant_controller() {
	println!("[merchant] enter");

	println!("The merchant greets you");

	loop {
		match *task::get_player_command().await.merchant().unwrap() {
			PlayerCommand::BuyItem(item) => {
				if task::consume_player_item_n(Item::Treasure, item_cost(item)).await {
					task::give_player_item(item).await
				} else {
					println!("You don't have enough treasure!");
				}
			}

			PlayerCommand::SellItem(item) => {
				if task::consume_player_item(item).await {
					task::give_player_item_n(Item::Treasure, item_cost(item)).await
				} else {
					println!("Try selling something you actually have lmao");
				}
			}

			PlayerCommand::Leave => {
				println!("The merchant tells you not to let the door hit you on the way out");
				break
			}
		}
	}

	println!("[merchant] leave");
}