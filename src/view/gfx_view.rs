mod window;
mod gfx;
mod util;
mod click_region;

mod map_view;
mod player_view;
mod battle_view;
mod merchant_view;

use crate::prelude::*;
use crate::gamestate::{GameState, GameCommand, Inventory};
use crate::task::{PlayerCommand, UntypedPromise, Promise, ControllerMode};
use super::{View, ViewCommand};

use util::*;

use map_view::MapView;
use player_view::PlayerView;
use battle_view::BattleView;
use merchant_view::MerchantView;

use click_region::ClickRegionEvent;
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

	camera_proj_view: Mat4,
	camera_forward: Vec3,

	map_view: MapView,
	player_view: PlayerView,
	battle_view: BattleView,
	merchant_view: MerchantView,
}


impl GfxView {
	pub fn new() -> GfxView {
		let window = window::Window::new().expect("Failed to create window");
		let mut gfx = Gfx::new();

		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
		}

		let shader = gfx.core().new_shader(
			include_str!("gfx_view/vert.glsl"),
			include_str!("gfx_view/frag.glsl"),
			&["a_vertex", "a_color"]
		);
		gfx.core().use_shader(shader);

		let map_view = MapView::new(&mut gfx);
		let player_view = PlayerView::new(&mut gfx);
		let battle_view = BattleView::new();
		let merchant_view = MerchantView::new();

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

			camera_proj_view: Mat4::ident(),
			camera_forward: Vec3::from_y(-1.0),

			map_view,
			player_view,
			battle_view,
			merchant_view,
		}
	}

	fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.unwrap_or(ControllerMode::Main)
	}

	fn process_move(&mut self, pos: Vec2) {
		let screen_pos = window_to_screen(self.window.size(), pos);

		let near_plane_pos = self.camera_proj_view.inverse() * screen_pos.extend(0.0).extend(1.0);
		let near_plane_pos = near_plane_pos.to_vec3() / near_plane_pos.w;

		let world_pos = intersect_ground(near_plane_pos, self.camera_forward);
		let ui_pos = screen_pos;

		let event = ClickRegionEvent::new_move(ui_pos, world_pos);

		let _ = match self.current_controller_mode() {
			ControllerMode::Main => self.map_view.process_mouse_event(event),
			ControllerMode::Battle => self.battle_view.process_mouse_event(event),
			ControllerMode::Merchant => self.merchant_view.process_mouse_event(event),
		};
	}

	fn process_click(&mut self, pos: Vec2) {
		let screen_pos = window_to_screen(self.window.size(), pos);

		let near_plane_pos = self.camera_proj_view.inverse() * screen_pos.extend(0.0).extend(1.0);
		let near_plane_pos = near_plane_pos.to_vec3() / near_plane_pos.w;

		let world_pos = intersect_ground(near_plane_pos, self.camera_forward);
		let ui_pos = screen_pos;

		let event = ClickRegionEvent::new_click(ui_pos, world_pos);

		let cmd = match self.current_controller_mode() {
			ControllerMode::Main => self.map_view.process_mouse_event(event),
			ControllerMode::Battle => self.battle_view.process_mouse_event(event),
			ControllerMode::Merchant => self.merchant_view.process_mouse_event(event),
		};

		if let Some(cmd) = cmd {
			self.push_player_command(cmd);
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
					self.process_move(self.mouse_pos);
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

	fn run_command(&mut self, gamestate: &GameState, cmd: ViewCommand, promise: UntypedPromise) {
		match cmd {
			ViewCommand::GetPlayerCommand => {
				self.map_view.on_awaiting_player_command();

				let promise = promise.player_command();

				if self.player_commands.is_empty() {
					let prev = self.player_command_promise.replace(promise);
					assert!(prev.is_none(), "Trying to queue two GetPlayerCommands");

				} else {
					promise.fulfill(self.player_commands.remove(0));
				}
			}

			ViewCommand::ShowMap { .. /*whole_map*/ } => {
				print_map(gamestate);
				promise.void().fulfill(());
			}

			ViewCommand::ShowInventory => {
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
						let world_loc = location_to_world(gamestate.player.location);

						self.camera_target = world_loc.to_x0z();

						self.map_view.on_player_move(gamestate);
						self.player_view.on_player_move(gamestate.player.location, promise.void());
						return;
					}

					_ => {}
				}

				promise.void().fulfill(());
			}

			ViewCommand::PushControllerMode(mode) => {
				self.controller_mode_stack.push(mode);
				println!("[view] mode transition -> {:?}", self.controller_mode_stack);

				self.map_view.on_mode_change(mode);
				self.battle_view.on_mode_change(mode);
				self.merchant_view.on_mode_change(mode);

				promise.void().fulfill(());
			}

			ViewCommand::PopControllerMode => {
				self.controller_mode_stack.pop();
				println!("[view] mode transition {:?} <-", self.controller_mode_stack);

				let current_ctl = self.current_controller_mode();
				self.map_view.on_mode_change(current_ctl);
				self.battle_view.on_mode_change(current_ctl);
				self.merchant_view.on_mode_change(current_ctl);

				promise.void().fulfill(());
			}
		}
	}
}

impl View for GfxView {
	fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	fn init(&mut self, gamestate: &GameState) {
		self.map_view.init(gamestate);
	}

	fn update(&mut self, gamestate: &GameState) {
		self.process_events();

		let commands = std::mem::replace(&mut self.commands, Vec::new());
		for (cmd, promise) in commands {
			self.run_command(gamestate, cmd, promise);
		}

		self.timer += 1.0/60.0;

		// TODO: refactor all of this into a Camera type
		self.camera_pos += (self.camera_target - self.camera_pos) / 60.0;

		let window_size = self.window.size();
		let aspect = window_size.x as f32 / window_size.y as f32;

		let projection = Mat4::ortho_aspect(1.2, aspect, -100.0, 200.0);
		let orientation = Quat::new(Vec3::from_y(1.0), PI/8.0 + self.timer.sin() as f32*0.02)
			* Quat::new(Vec3::from_x(1.0), -PI/6.0);

		let translation = Mat4::translate(-self.camera_pos + orientation.forward() * 3.0);
		self.camera_forward = orientation.forward();

		self.camera_proj_view = projection * orientation.conjugate().to_mat4() * translation;
		let ui_proj_view = Mat4::ortho_aspect(1.0, aspect, -100.0, 200.0);

		self.gfx.core().set_viewport(window_size);

		self.gfx.core().set_bg_color(Color::grey(0.1));
		self.gfx.core().clear();
		self.gfx.ui().clear();

		self.gfx.core().set_uniform_mat4("u_proj_view", &self.camera_proj_view);
		self.map_view.render(&mut self.gfx, gamestate);
		self.battle_view.render(&mut self.gfx, gamestate);
		self.merchant_view.render(&mut self.gfx, gamestate);
		self.player_view.render(&mut self.gfx, gamestate);

		self.gfx.draw_world_ui();

		self.gfx.core().set_uniform_mat4("u_proj_view", &ui_proj_view);
		self.gfx.draw_screen_ui();

		self.window.swap();
	}

	fn should_quit(&self) -> bool { self.should_quit }
}


fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", super::text_view::util::render_map(&state, state.map.bounds()));
	println!("=============");
}



fn window_to_screen(window_size: Vec2i, pos: Vec2) -> Vec2 {
	let window_half = window_size.to_vec2() / 2.0;
	(pos - window_half) / window_half * Vec2::new(1.0, -1.0)
}