mod window;
mod gfx;
mod vertex;

mod map_view;

use crate::prelude::*;
use crate::game_state::{GameState, GameCommand, Inventory};
use crate::task::{PlayerCommand, UntypedPromise, Promise, ControllerMode};
use super::{View, ViewCommand};

use gfx::Gfx;


pub struct GfxView {
	commands: Vec<(ViewCommand, UntypedPromise)>,
	controller_mode_stack: Vec<ControllerMode>,

	player_commands: Vec<PlayerCommand>,
	player_command_promise: Option<Promise<PlayerCommand>>,

	window: window::Window,
	gfx: Gfx,

	should_quit: bool,

	map_view: map_view::MapView,
}


impl GfxView {
	pub fn new() -> GfxView {
		let window = window::Window::new().expect("Failed to create window");
		let mut gfx = Gfx::new();

		let shader = gfx.new_shader(
			include_str!("gfx_view/vert.glsl"),
			include_str!("gfx_view/frag.glsl"),
			&["a_vertex", "a_color"]
		);
		gfx.use_shader(shader);

		let map_view = map_view::MapView::new(&mut gfx);

		GfxView {
			commands: Vec::new(),
			controller_mode_stack: Vec::new(),

			player_commands: Vec::new(),
			player_command_promise: None,

			window,
			gfx,

			should_quit: false,

			map_view,
		}
	}

	fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.expect("Empty controller stack!")
	}

	fn process_events(&mut self) {
		use crate::controller::main::PlayerCommand::*;
		use sdl2::keyboard::Keycode;

		let mut event_pump = self.window.event_pump();
		for event in event_pump.poll_iter() {
			use sdl2::event::Event;

			match event {
				Event::Quit{..} => { self.should_quit = true; }
				Event::KeyDown{ keycode: Some(keycode), ..} => match keycode {
					Keycode::Escape => self.push_player_command(PlayerCommand::Main(Quit)),

					Keycode::W => self.push_player_command(PlayerCommand::Main(GoNorth)),
					Keycode::S => self.push_player_command(PlayerCommand::Main(GoSouth)),
					Keycode::D => self.push_player_command(PlayerCommand::Main(GoEast)),
					Keycode::A => self.push_player_command(PlayerCommand::Main(GoWest)),

					_ => {}
				}
				_ => {}
			}
		}
	}

	fn push_player_command(&mut self, cmd: PlayerCommand) {
		if let Some(promise) = self.player_command_promise.take() {
			promise.fulfill(cmd);
		} else {
			self.player_commands.push(cmd);
		}
	}
}

impl View for GfxView {
	fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	fn update(&mut self, game_state: &GameState) {
		self.process_events();

		let commands = std::mem::replace(&mut self.commands, Vec::new());

		for (cmd, promise) in commands {
			match cmd {
				ViewCommand::GetPlayerCommand => {
					let promise = promise.player_command();

					if self.player_commands.is_empty() {
						let prev = self.player_command_promise.replace(promise);
						assert!(prev.is_none(), "Trying to queue two GetPlayerCommands");

					} else {
						promise.fulfill(self.player_commands.remove(0));
					}
				}

				ViewCommand::ShowMap { .. /*whole_map*/ } => {
					print_map(game_state);
					promise.void().fulfill(());
				}

				ViewCommand::ShowInventory => {
					promise.void().fulfill(());
				}

				ViewCommand::GameCommand(event) => {
					use crate::game_state::HealthModifyReason;
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
									if game_state.player.is_dead() {
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

		self.gfx.set_bg_color(Color::grey(0.1));
		self.gfx.clear();

		self.map_view.render(&mut self.gfx, game_state);

		self.window.swap();
	}

	fn should_quit(&self) -> bool { self.should_quit }
}


fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", super::text_view::util::render_map(&state, state.map.bounds()));
	println!("=============");
}