pub mod render_buffer;
pub mod util;

use crate::prelude::*;
use crate::game_state::{GameState, GameCommand};
use crate::task::{PlayerCommand, UntypedPromise, ControllerMode};


pub struct View {
	commands: Vec<(ViewCommand, UntypedPromise)>,
	controller_mode_stack: Vec<ControllerMode>,
}

#[derive(Copy, Clone)]
pub enum ViewCommand {
	GetPlayerCommand,
	ShowMap { whole_map: bool },
	GameCommand(GameCommand),
	PushControllerMode(ControllerMode),
	PopControllerMode,
}


impl View {
	pub fn new() -> View {
		View {
			commands: Vec::new(),
			controller_mode_stack: Vec::new(),
		}
	}

	pub fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	pub fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.expect("Empty controller stack!")
	}

	pub fn update(&mut self, game_state: &GameState) {
		let commands = std::mem::replace(&mut self.commands, Vec::new());

		for (cmd, promise) in commands {
			match cmd {
				ViewCommand::GetPlayerCommand => {
					let command = get_player_command_sync(self.current_controller_mode());
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
					use crate::game_state::{Item, HealthModifyReason};
					use std::cmp::Ordering;

					match event {
						GameCommand::GivePlayerItem(item, _) => match item {
							Item::Food => println!("You found food!"),
							Item::Treasure => println!("You found treasure!"),
							Item::Key => println!("You found a key!"),
							Item::Map => {
								// TODO: println!("You found another map. It may have some value");
								// how do I find out if player already had a map?
								println!("You found a map!");
							}
						}

						GameCommand::ModifyPlayerHealth(n, reason) => match n.cmp(&0) {
							Ordering::Greater => {
								println!("You gained {} health", n);
							}

							Ordering::Less => match reason {
								HealthModifyReason::Attack => {
									println!("You lost {} health!", -n);
								}

								HealthModifyReason::Hunger => {
									println!("Your hunger cost you {} health.", -n);
								}

								_ => {}
							}

							Ordering::Equal => {}
						}

						GameCommand::MovePlayer(dir) => {
							println!("You move {}", dir);
						}

						_ => {}
					}

					promise.void().fulfill(());
				}

				ViewCommand::PushControllerMode(mode) => {
					self.controller_mode_stack.push(mode);
					println!("[view] mode transition -> {:?}", self.controller_mode_stack);

					promise.void().fulfill(());
				}

				ViewCommand::PopControllerMode => {
					self.controller_mode_stack.pop();
					println!("[view] mode transition {:?} <-", self.controller_mode_stack);

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

fn get_player_command_sync(controller_mode: ControllerMode) -> PlayerCommand {
	use std::io::{Write, BufRead};
	use ControllerMode::*;

	loop {
		print!("> ");

		std::io::stdout().flush()
			.expect("Failed to flush");

		let mut command = std::io::stdin().lock()
			.lines().next()
			.expect("EOF")
			.expect("Failed to read stdin");

		if command.is_empty() {
			continue;
		}

		command.make_ascii_lowercase();

		if command.starts_with("d ") {
			let parts = command[2..]
				.split_whitespace()
				.map(str::to_owned)
				.collect();

			break PlayerCommand::Debug(parts)
		}

		if let Some(command) = match controller_mode {
			Main => parse_main_player_command(&command),
			Battle => parse_battle_player_command(&command),
			Merchant => parse_merchant_player_command(&command),
		} {
			break command
		}

		println!("what does '{}' mean??", command);
	}
}


fn parse_main_player_command(cmd: &str) -> Option<PlayerCommand> {
	use crate::controller::main::PlayerCommand::*;

	let cmd = match cmd {
		"n" | "north" => GoNorth,
		"e" | "east" => GoEast,
		"s" | "south" => GoSouth,
		"w" | "west" => GoWest,
		"m" | "map" => ShowMap,

		"heal" | "eat" => Heal,

		// "r" | "restart" => Some(Event::Restart),
		"q" | "quit" => Quit,
		_ => return None
	};


	Some(PlayerCommand::Main(cmd))
}

fn parse_battle_player_command(cmd: &str) -> Option<PlayerCommand> {
	use crate::controller::battle::PlayerCommand::*;

	let cmd = match cmd {
		"f" | "fight" => Attack,
		"e" | "eat" | "h" | "heal" => Heal,
		"r" | "run" | "flee" => Flee,
		_ => return None
	};

	Some(PlayerCommand::Battle(cmd))
}

fn parse_merchant_player_command(cmd: &str) -> Option<PlayerCommand> {
	use crate::controller::merchant::PlayerCommand::*;
	use crate::game_state::Item;

	let cmd = match cmd {
		"b food" => BuyItem(Item::Food),
		"b map" => BuyItem(Item::Map),
		"b key" => BuyItem(Item::Key),

		"s food" => SellItem(Item::Food),
		"s map" => SellItem(Item::Map),
		"s key" => SellItem(Item::Key),

		"l" | "leave" => Leave,

		_ => return None
	};


	Some(PlayerCommand::Merchant(cmd))
}