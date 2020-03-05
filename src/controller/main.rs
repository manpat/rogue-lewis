use crate::prelude::*;
use crate::controller::*;
use crate::game_state::GameState;

pub struct MainController;

impl Controller for MainController {
	fn init(&mut self, state: &GameState) {
		println!("Which way do you go?");
		self.print_local_area(state);
	}

	fn run_command(&mut self, state: &mut GameState, command: &str) -> Event {
		match command {
			"n" | "north" => self.try_move(state, Direction::North),
			"e" | "east" => self.try_move(state, Direction::East),
			"s" | "south" => self.try_move(state, Direction::South),
			"w" | "west" => self.try_move(state, Direction::West),
			"m" | "map" => self.print_map(state),

			"h" | "help" => self.print_help(),

			"testbattle" => return Event::Transition(box BattleController),
			"testmerchant" => return Event::Transition(box MerchantController),

			"iwin" => return Event::Win,
			"ilose" => return Event::Lose,

			"r" | "restart" => return Event::Restart,
			"q" | "quit" => return Event::Quit,
			_ => println!("what now?")
		}

		Event::Continue
	}
}

impl MainController {
	fn try_move(&self, state: &mut GameState, dir: Direction) {
		if state.try_move_player(dir) {
			println!("You move {}", dir);
			self.print_local_area(state);
		} else {
			println!("You can't go that way");
		}
	}

	fn print_map(&self, state: &GameState) {
		println!("==== map ====");
		println!("{}", crate::rendering::render_map(&state, state.map.bounds()));
		println!("=============");
	}

	fn print_local_area(&self, state: &GameState) {
		let bounds = state.map.iter()
			.filter(|(loc, _)| loc.distance(state.player_location) < 2)
			.fold(Bounds::empty(), |bounds, (loc, _)| bounds.include(loc))
			.expand(1, 0);

		println!("=============");
		println!("{}", crate::rendering::render_map(&state, bounds));
		println!("=============");
	}

	fn print_help(&self) {
		println!("pls implement help");
	}
}