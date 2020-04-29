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
	mouse_pos: Vec2,

	timer: f64,
	camera_target: Vec3,
	camera_pos: Vec3,

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
			mouse_pos: Vec2::zero(),

			timer: 0.0,
			camera_target: Vec3::zero(),
			camera_pos: Vec3::zero(),

			map_view,
		}
	}

	fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.expect("Empty controller stack!")
	}

	fn process_click(&mut self, pos: Vec2) {
		let window_half = self.window.size().to_vec2() / 2.0;
		let diff = (pos - window_half) / window_half * Vec2::new(1.0, -1.0);

		use crate::controller::main::PlayerCommand::*;

		if diff.y > 0.6 {
			self.push_player_command(PlayerCommand::Main(GoNorth));
		} else if diff.y < -0.6 {
			self.push_player_command(PlayerCommand::Main(GoSouth));
		}

		if diff.x > 0.6 {
			self.push_player_command(PlayerCommand::Main(GoEast));
		} else if diff.x < -0.6 {
			self.push_player_command(PlayerCommand::Main(GoWest));
		}
	}

	fn process_events(&mut self) {
		use crate::controller::main::PlayerCommand::*;

		use glutin::{WindowEvent, ElementState::Pressed, MouseButton::Left as LeftMouse};
		use glutin::dpi::PhysicalPosition;

		let events = self.window.poll_events();

		for event in events {
			match event {
				WindowEvent::CursorMoved {position, ..} => {
					let PhysicalPosition{x, y} = position.to_physical(self.window.dpi());
					self.mouse_pos = Vec2::new(x as f32, y as f32);
				}

				WindowEvent::MouseInput {state: Pressed, button: LeftMouse, ..} => {
					self.process_click(self.mouse_pos);
				}

				WindowEvent::CloseRequested => self.push_player_command(PlayerCommand::Main(Quit)),

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
							self.camera_target = game_state.player.location
								.to_vec2i().to_vec2()
								.to_x0z() * Vec3::new(3.0, 0.0,-3.0);
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

		self.timer += 1.0/60.0;

		self.camera_pos += (self.camera_target - self.camera_pos) / 60.0;

		let window_size = self.window.size();
		let aspect = window_size.x as f32 / window_size.y as f32;

		let projection = Mat4::ortho_aspect(8.0, aspect, -100.0, 200.0);
		let orientation = Quat::new(Vec3::from_y(1.0), PI/8.0 + self.timer.sin() as f32*0.02)
			* Quat::new(Vec3::from_x(1.0), -PI/6.0);

		let translation = Mat4::translate(-self.camera_pos + orientation.forward() * 3.0);

		let proj_view = projection * orientation.conjugate().to_mat4() * translation;

		self.gfx.set_uniform_mat4("u_proj_view", &proj_view);
		self.gfx.set_viewport(window_size);

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