pub mod render_buffer;
pub mod util;

use crate::prelude::*;
use crate::game_state::GameState;
use crate::task::{self, Coordinator, PlayerCommand, ControllerEvent};


pub struct View {
	coordinator: Coordinator,
	show_map_request: Option<bool>,
	player_command_request: bool,
}


pub enum ViewEvent {
	PlayerCommand(task::PlayerCommand),
	MapShown,
	ModeChanged,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ViewEventType {
	PlayerCommand,
	MapShown,
	ModeChanged,
}

impl ViewEvent {
	pub fn to_type(&self) -> ViewEventType {
		match self {
			ViewEvent::PlayerCommand(_) => ViewEventType::PlayerCommand,
			ViewEvent::MapShown => ViewEventType::MapShown,
			ViewEvent::ModeChanged => ViewEventType::ModeChanged,
		}
	}
}


impl View {
	pub fn new(coordinator: Coordinator) -> View {
		View {
			coordinator,
			show_map_request: None,
			player_command_request: false,
		}
	}

	pub fn request_show_map(&mut self, whole_map: bool) {
		self.show_map_request = Some(whole_map);
	}

	pub fn request_player_command(&mut self) {
		self.player_command_request = true;
	}

	pub fn notify_controller_event(&mut self, event: ControllerEvent) {
		use crate::game_state::{GameState, Item};

		match event {
			// TODO: this should be done in render
			ControllerEvent::PlayerGotItem(item) => match item {
				Item::Food => println!("You found food!"),
				Item::Treasure => println!("You found treasure!"),
				Item::Key => println!("You found a key!"),
				Item::Map => {
					// TODO: println!("You found another map. It may have some value");
					// how do I find out if player already had a map?
					println!("You found a map!");
				}
			}
		}
	}

	pub fn render(&mut self, game_state: &GameState) {
		if let Some(whole_map) = self.show_map_request.take() {
			if whole_map {
				print_map(game_state);
			} else {
				print_local_area(game_state);
			}

			self.coordinator.notify_view_event(ViewEvent::MapShown);
		}

		if self.player_command_request {
			self.player_command_request = false;
			let command = get_player_command_sync();
			self.coordinator.notify_view_event(ViewEvent::PlayerCommand(command));
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