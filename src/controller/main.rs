use crate::prelude::*;
use crate::controller::*;
use crate::game_state::{GameState, Item};
use crate::room::EncounterType;

#[derive(Debug)]
pub struct MainController;

impl Controller for MainController {
	fn enter(&mut self, state: &mut GameState) {
		println!("Which way do you go?");
		print_local_area(state);
	}

	fn run_command(&mut self, state: &mut GameState, command: &str) -> Option<Event> {
		match command {
			"n" | "north" => try_move(state, Direction::North),
			"e" | "east" => try_move(state, Direction::East),
			"s" | "south" => try_move(state, Direction::South),
			"w" | "west" => try_move(state, Direction::West),
			"m" | "map" => { print_map(state); None },

			"h" | "help" => { print_help(); None },

			// TODO: eat | heal

			"testbattle" => {
				let loc = state.player.location;
				state.spawn_enemy_at(loc, random());
				Some(Event::Enter(box BattleController::new(loc)))
			},
			"testmerchant" => Some(Event::Enter(box MerchantController {})),

			"iwin" => Some(Event::Win),
			"ilose" => Some(Event::Lose),

			"r" | "restart" => Some(Event::Restart),
			"q" | "quit" => Some(Event::Quit),
			_ => {
				println!("what now?");
				None
			}
		}
	}
}

fn try_move(state: &mut GameState, dir: Direction) -> Option<Event> {
	if !state.try_move_player(dir) {
		println!("You can't go that way");
		return None
	}

	println!("You move {}", dir);

	if !state.player.inventory.take(Item::Food) {
		state.player.hunger -= 1;
		if state.player.hunger <= 0 {
			println!("You starve to death");
			return Some(Event::Lose)
		} else {
			println!("You have run out of food! You can travel {} rooms", state.player.hunger);
		}
	} else {
		state.player.hunger = 10;
	}

	let player_pos = state.player.location;
	let current_room = state.map.get(player_pos).unwrap();

	state.map.mark_visited(player_pos);

	if let Some(encounter_ty) = current_room.encounter {
		let encounter_event = run_encounter(state, encounter_ty);

		if !encounter_ty.is_persistent() {
			state.remove_encounter_at(player_pos);
		}

		if encounter_event.is_some() {
			return encounter_event;
		}
	}

	print_local_area(state);

	None
}

fn run_encounter(state: &mut GameState, encounter_ty: EncounterType) -> Option<Event> {
	println!("]]] running encounter {:?}", encounter_ty);

	let inv = &mut state.player.inventory;
	let player_loc = state.player.location;

	match encounter_ty {
		EncounterType::Food => {
			inv.add(Item::Food);
			println!("You found food");
		}

		EncounterType::Treasure => {
			inv.add(Item::Treasure);
			println!("You found treasure");
		}

		EncounterType::Key => {
			inv.add(Item::Key);
			println!("You found a key!");
		}

		EncounterType::Map => {
			if !inv.has(Item::Map) {
				println!("You found a map!");
			} else {
				println!("You found another map. It may have some value");
			}

			inv.add(Item::Map);
		}

		EncounterType::Chest => {
			let chest_items = [Item::Food, Item::Treasure, Item::Key];

			if inv.take(Item::Key) {
				let num_items = rng().gen_range(1, 5);
				let items = chest_items.choose_multiple(&mut rng(), num_items);

				println!("You found a chest!");
				println!("You open it with one of your keys");

				for item in items {
					println!("You found a {:?}", item);
					inv.add(*item);
				}

			} else {
				println!("You found a chest, but don't have a key to open it");
			}
		}

		EncounterType::Monster => {
			if state.get_enemy(player_loc).is_none() {
				state.spawn_enemy_at(player_loc, false);
			}

			return Some(Event::Enter(box BattleController::new(player_loc)))
		}

		EncounterType::Boss => {
			if state.get_enemy(player_loc).is_none() {
				state.spawn_enemy_at(player_loc, true);
			}

			return Some(Event::Enter(box BattleController::new(player_loc)))
		}

		EncounterType::Merchant => {
			return Some(Event::Enter(box MerchantController {}))
		}		

		_ => {}
	}

	None
}

fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", crate::rendering::render_map(&state, state.map.bounds()));
	println!("=============");
}

fn print_local_area(state: &GameState) {
	let bounds = state.map.iter()
		.filter(|(loc, _)| loc.distance(state.player.location) < 2)
		.fold(Bounds::empty(), |bounds, (loc, _)| bounds.include(loc))
		.expand(1, 0);

	println!("=============");
	println!("{}", crate::rendering::render_map(&state, bounds));
	println!("=============");
}

fn print_help() {
	println!("pls implement help");
}