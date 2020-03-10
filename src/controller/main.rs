use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;
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
	if state.try_move_player(dir) {
		println!("You move {}", dir);

		let player_pos = state.player.location;
		let current_room = state.map.get_mut(player_pos).unwrap();

		if current_room.encounter.is_some() {
			if let Some(event) = run_encounter(&mut state.player, current_room) {
				return Some(event);
			}
		}

		print_local_area(state);

	} else {
		println!("You can't go that way");
	}

	None
}

fn run_encounter(player: &mut crate::game_state::Player, room: &mut crate::room::Room) -> Option<Event> {
	use crate::game_state::Item;

	let encounter = if let Some(e) = room.encounter { e } else { return None };

	println!("]]] running encounter {:?}", encounter);

	match encounter {
		EncounterType::Food => {
			player.inventory.add(Item::Food);
			room.encounter = None;

			println!("You found food");
		}

		EncounterType::Treasure => {
			player.inventory.add(Item::Treasure);
			room.encounter = None;

			println!("You found treasure");
		}

		EncounterType::Key => {
			player.inventory.add(Item::Key);
			room.encounter = None;

			println!("You found a key!");
		}

		EncounterType::Map => {
			if !player.inventory.has(Item::Map) {
				player.inventory.add(Item::Map);
				room.encounter = None;

				println!("You found a map!");
			} else {
				println!("You found a map, but you already have one so you leave it");
			}
		}

		EncounterType::Chest => {
			if player.inventory.take(Item::Key) {
				room.encounter = None;

				let item = Item::Food; // TODO

				println!("You found a chest!");
				println!("You open it with one of your keys to receive {:?}", item);

				player.inventory.add(item);
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