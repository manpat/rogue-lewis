pub mod render_buffer;
pub mod util;

use crate::prelude::*;
use crate::game_state::GameState;
use crate::task::{self, Coordinator, PlayerCommand, GameCommand, UntypedPromise};


pub struct View {
	coordinator: Coordinator,
	commands: Vec<(ViewCommand, UntypedPromise)>,
}

#[derive(Copy, Clone)]
pub enum ViewCommand {
	GetPlayerCommand,
	ShowMap { whole_map: bool },
	GameCommand(GameCommand),
}


impl View {
	pub fn new(coordinator: Coordinator) -> View {
		View {
			coordinator,
			commands: Vec::new(),
		}
	}

	pub fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	pub fn render(&mut self, game_state: &GameState) {
		for (cmd, promise) in self.commands.drain(..) {
			match cmd {
				ViewCommand::GetPlayerCommand => {
					let command = get_player_command_sync();
					promise.player_command().fulfill(command);
				}

				ViewCommand::ShowMap { whole_map } => {
					if whole_map {
						print_map(game_state);
					} else {
						print_local_area(game_state);
					}

					promise.void().fulfill(());
				}

				ViewCommand::GameCommand(event) => {
					use crate::game_state::{GameState, Item};

					match event {
						GameCommand::GivePlayerItem(item) => match item {
							Item::Food => println!("You found food!"),
							Item::Treasure => println!("You found treasure!"),
							Item::Key => println!("You found a key!"),
							Item::Map => {
								// TODO: println!("You found another map. It may have some value");
								// how do I find out if player already had a map?
								println!("You found a map!");
							}
						}

						_ => {}
					}

					promise.void().fulfill(());
				}
			}
		}
	}
}

fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", util::render_map(&state, state.map.bounds()));
	println!("=============");
}

fn print_local_area(state: &GameState) {
	let bounds = state.map.iter()
		.filter(|(loc, _)| loc.distance(state.player.location) < 2)
		.fold(Bounds::empty(), |bounds, (loc, _)| bounds.include(loc))
		.expand(1, 0);

	println!("=============");
	println!("{}", util::render_map(&state, bounds));
	println!("=============");
}

fn get_player_command_sync() -> PlayerCommand {
	use std::io::{Write, BufRead};

	loop {
		print!("> ");

		std::io::stdout().flush()
			.expect("Failed to flush");

		let mut command = std::io::stdin().lock()
			.lines().next()
			.expect("EOF")
			.expect("Failed to read stdin");


		if !command.is_empty() {
			command.make_ascii_lowercase();
			break PlayerCommand(command)
		}
	}
}