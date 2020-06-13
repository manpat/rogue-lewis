pub mod render_buffer;
pub mod util;

use crate::prelude::*;
use crate::gamestate::{GameState, GameCommand, Inventory};
use crate::task::{PlayerCommand, UntypedPromise, ControllerMode};
use super::{View, ViewCommand};


pub struct TextView {
	commands: Vec<(ViewCommand, UntypedPromise)>,
	controller_mode_stack: Vec<ControllerMode>,
}


impl TextView {
	pub fn new() -> TextView {
		TextView {
			commands: Vec::new(),
			controller_mode_stack: Vec::new(),
		}
	}

	fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.expect("Empty controller stack!")
	}
}

impl View for TextView {
	fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	fn update(&mut self, gamestate: &GameState) {
		let commands = std::mem::replace(&mut self.commands, Vec::new());

		for (cmd, promise) in commands {
			match cmd {
				ViewCommand::GetPlayerCommand => {
					let command = get_player_command_sync(self.current_controller_mode());
					promise.player_command().fulfill(command);
				}

				ViewCommand::ShowMap { whole_map } => {
					if whole_map {
						print_map(gamestate);
					} else {
						print_local_area(gamestate);
					}

					promise.void().fulfill(());
				}

				ViewCommand::ShowInventory => {
					print_inventory(&gamestate.player.inventory);
					promise.void().fulfill(());
				}

				ViewCommand::GameCommand(event) => {
					use crate::gamestate::HealthModifyReason;
					use crate::item::Item;
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

							Item::Potion => println!("You found a potion"),

							Item::Equipment(e) => {
								// TODO: a/an obvs doesn't work
								println!("You found a {:?}", e);
							}
						}

						GameCommand::ModifyPlayerHealth(n, reason) => match n.cmp(&0) {
							Ordering::Greater => {
								println!("You gained {} health", n);
							}

							Ordering::Less => match reason {
								HealthModifyReason::Attack => {
									println!("You lost {} health!", -n);
									if gamestate.player.is_dead() {
										println!("Unfortunately, the strike is fatal");
									}
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

	fn should_quit(&self) -> bool { false }
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

fn print_inventory(inv: &Inventory) {
	use crate::item::Item;

	let mut items_sorted = inv.iter_items()
		.collect::<Vec<_>>();

	items_sorted.sort();

	let mut items_n_counts = Vec::new();
	let mut items = &items_sorted[..];

	while !items.is_empty() {
		let item = items[0];
		let partition_point = items.iter().position(|&i| i != item).unwrap_or(items.len());
		let (matching, tail) = items.split_at(partition_point);

		items_n_counts.push((item, matching.len()));
		items = tail;
	}

	items_n_counts.sort_by_key(|(_, count)| *count);

	let items_str = items_n_counts.into_iter()
		.map(|(i, count)| match i {
			Item::Equipment(e) => format!("{:?} (x{})", e, count),
			i => format!("{:?} (x{})", i, count)
		})
		.collect::<Vec<_>>()
		.join(", ");

	if !items_str.is_empty() {
		println!("Items: {}", items_str);
	}

	println!("Treasure: {}", inv.count(Item::Treasure));
	println!("Food: {}", inv.count(Item::Food));
}

fn get_player_command_sync(controller_mode: ControllerMode) -> PlayerCommand {
	use std::io::{Write, BufRead};
	use ControllerMode::*;

	loop {
		print!("> ");

		std::io::stdout().flush()
			.expect("Failed to flush");

		let mut command_str = std::io::stdin().lock()
			.lines().next()
			.expect("EOF")
			.expect("Failed to read stdin");

		if command_str.is_empty() {
			continue;
		}

		command_str.make_ascii_lowercase();

		if command_str.starts_with("d ") {
			let parts = command_str[2..]
				.split_whitespace()
				.map(str::to_owned)
				.collect();

			break PlayerCommand::Debug(parts)
		}

		if let Some(command) = match controller_mode {
			Main => parse_main_player_command(&command_str),
			Battle => parse_battle_player_command(&command_str),
			Merchant => parse_merchant_player_command(&command_str),
		} {
			break command
		}

		println!("what does '{}' mean??", command_str);
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
		"i" | "inv" | "inventory" => ShowInventory,

		"heal" | "eat" => Heal,
		"use" | "interact" => Interact,

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
	use crate::item::Item;

	let cmd = match cmd {
		"b food" => BuyItem(Item::Food),
		"b map" => BuyItem(Item::Map),
		"b key" => BuyItem(Item::Key),
		"b equipment" => BuyItem(Item::Equipment(random())),

		"s food" => SellItem(Item::Food),
		"s map" => SellItem(Item::Map),
		"s key" => SellItem(Item::Key),

		"l" | "leave" => Leave,

		_ => return None
	};


	Some(PlayerCommand::Merchant(cmd))
}