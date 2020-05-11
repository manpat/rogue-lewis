mod window;
mod gfx;
mod vertex;
mod mesh_builder;
mod util;

mod map_view;
mod player_view;
mod click_region_view;

use crate::prelude::*;
use crate::gamestate::{GameState, GameCommand, Inventory};
use crate::task::{PlayerCommand, UntypedPromise, Promise, ControllerMode};
use super::{View, ViewCommand};

use util::*;

use map_view::MapView;
use player_view::PlayerView;
use click_region_view::ClickRegionView;

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
	click_region_view: ClickRegionView,
}


impl GfxView {
	pub fn new() -> GfxView {
		let window = window::Window::new().expect("Failed to create window");
		let mut gfx = Gfx::new();

		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
		}

		let shader = gfx.new_shader(
			include_str!("gfx_view/vert.glsl"),
			include_str!("gfx_view/frag.glsl"),
			&["a_vertex", "a_color"]
		);
		gfx.use_shader(shader);

		let map_view = MapView::new(&mut gfx);
		let player_view = PlayerView::new(&mut gfx);

		let mut click_region_view = ClickRegionView::new(&mut gfx);
		click_region_view.gen_regions_for_room(Vec2::zero());

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
			click_region_view,
		}
	}

	fn current_controller_mode(&self) -> ControllerMode {
		self.controller_mode_stack.last()
			.cloned()
			.expect("Empty controller stack!")
	}

	fn process_move(&mut self, pos: Vec2) {
		let window_half = self.window.size().to_vec2() / 2.0;
		let screen_pos = (pos - window_half) / window_half * Vec2::new(1.0, -1.0);

		let screen_pos = self.camera_proj_view.inverse() * screen_pos.extend(0.0).extend(1.0);
		let screen_pos = screen_pos.to_vec3() / screen_pos.w;

		let world_pos = intersect_ground(screen_pos, self.camera_forward);

		self.click_region_view.process_hover(world_pos.to_xz());
	}

	fn process_click(&mut self, pos: Vec2) {
		let window_half = self.window.size().to_vec2() / 2.0;
		let screen_pos = (pos - window_half) / window_half * Vec2::new(1.0, -1.0);

		let screen_pos = self.camera_proj_view.inverse() * screen_pos.extend(0.0).extend(1.0);
		let screen_pos = screen_pos.to_vec3() / screen_pos.w;

		let world_pos = intersect_ground(screen_pos, self.camera_forward);

		let ctl_mode = self.current_controller_mode();

		if let Some(cmd) = self.click_region_view.process_click(world_pos.to_xz(), ctl_mode) {
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
						self.click_region_view.gen_regions_for_room(world_loc);

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

impl View for GfxView {
	fn submit_command(&mut self, cmd: ViewCommand, promise: UntypedPromise) {
		self.commands.push((cmd, promise));
	}

	fn update(&mut self, gamestate: &GameState) {
		self.process_events();

		let commands = std::mem::replace(&mut self.commands, Vec::new());
		for (cmd, promise) in commands {
			self.run_command(gamestate, cmd, promise);
		}

		self.timer += 1.0/60.0;

		self.camera_pos += (self.camera_target - self.camera_pos) / 60.0;

		let window_size = self.window.size();
		let aspect = window_size.x as f32 / window_size.y as f32;

		let projection = Mat4::ortho_aspect(2.0, aspect, -100.0, 200.0);
		let orientation = Quat::new(Vec3::from_y(1.0), PI/8.0 + self.timer.sin() as f32*0.02)
			* Quat::new(Vec3::from_x(1.0), -PI/6.0);

		let translation = Mat4::translate(-self.camera_pos + orientation.forward() * 3.0);
		self.camera_forward = orientation.forward();

		self.camera_proj_view = projection * orientation.conjugate().to_mat4() * translation;
		// let ui_proj_view = Mat4::ortho_aspect(1.0, aspect, -100.0, 200.0);

		self.gfx.set_viewport(window_size);

		self.gfx.set_bg_color(Color::grey(0.1));
		self.gfx.clear();

		self.gfx.set_uniform_mat4("u_proj_view", &self.camera_proj_view);
		self.map_view.render(&mut self.gfx, gamestate);

		self.click_region_view.render(&mut self.gfx);

		self.player_view.render(&mut self.gfx, gamestate);

		// self.gfx.set_uniform_mat4("u_proj_view", &ui_proj_view);

		self.window.swap();
	}

	fn should_quit(&self) -> bool { self.should_quit }
}


fn print_map(state: &GameState) {
	println!("==== map ====");
	println!("{}", super::text_view::util::render_map(&state, state.map.bounds()));
	println!("=============");
}




fn intersect_plane(plane_point: Vec3, plane_normal: Vec3, line_point: Vec3, line_direction: Vec3) -> Option<Vec3> {
	let line_direction = line_direction.normalize();

	if plane_normal.dot(line_direction).abs() < 0.01 {
		return None;
	}

	let t = (plane_normal.dot(plane_point) - plane_normal.dot(line_point)) / plane_normal.dot(line_direction);
	Some(line_point + line_direction * t)
}


fn intersect_ground(line_point: Vec3, line_direction: Vec3) -> Vec3 {
	let plane_point = Vec3::zero();
	let plane_normal = Vec3::from_y(1.0);

	intersect_plane(plane_point, plane_normal, line_point, line_direction)
		.expect("Camera forward perpendicular to ground plane")
}