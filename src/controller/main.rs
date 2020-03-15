use crate::prelude::*;
use crate::controller::*;
use crate::game_state::{GameState, Item};
use crate::room::EncounterType;

#[derive(Debug)]
pub struct MainController;

impl Controller for MainController {
	fn init(&mut self, state: &GameState) {
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

			"testbattle" => Some(Event::Transition(box BattleController { boss: random() })),
			"testmerchant" => Some(Event::Transition(box MerchantController {})),

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
			if inv.take(Item::Key) {
				let item = Item::Food; // TODO

				println!("You found a chest!");
				println!("You open it with one of your keys to receive {:?}", item);

				inv.add(item);
			} else {
				println!("You found a chest, but don't have a key to open it");
			}
		}

		EncounterType::Monster => {
			return Some(Event::Transition(box BattleController { boss: false }))
		}

		EncounterType::Boss => {
			return Some(Event::Transition(box BattleController { boss: true }))
		}

		EncounterType::Merchant => {
			return Some(Event::Transition(box MerchantController {}))
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