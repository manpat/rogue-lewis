#![feature(nll)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![deny(rust_2018_idioms, future_incompatible)]
#![allow(elided_lifetimes_in_paths)]

mod prelude;
mod task;
mod types;
mod game_state;
mod map;
mod room;
mod rendering;
mod controller;
mod enemy;

use prelude::*;
use game_state::GameState;

fn main() {
	let mut executor = task::Executor::new();

	let game_state = generate_game_state();
	let game_state = Rc::new(RefCell::new(game_state));
	let coordinator = task::Coordinator::new(Rc::clone(&game_state));

	executor.queue(controller::run_main_controller(coordinator.clone()));

	while executor.num_queued_tasks() > 0 {
		executor.poll();
		coordinator.run(&mut game_state.borrow_mut());

		// 	Some(Event::Restart) => {
		// 		println!("The walls warp and shift around you and your sense of reality temporarily disolves");
		// 		println!("You are unsure if any of the events you've experienced until now actually happened");
		// 		*ctl_ctx.state = generate_game_state();
		// 		let mut main_controller = Controller::from(MainController);
		// 		main_controller.enter(ctl_ctx);
		// 		controllers = vec![main_controller];
		// 	}
	}
}


fn generate_game_state() -> GameState {
	let mut state = GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}