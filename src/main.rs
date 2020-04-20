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
mod view;
mod controller;
mod enemy;

use prelude::*;
use game_state::GameState;

fn main() {
	let mut executor = task::Executor::new();

	let game_state = generate_game_state();
	let game_state = Rc::new(RefCell::new(game_state));
	let mut coordinator = task::Coordinator::new(Rc::clone(&game_state));

	unsafe {
		COORDINATOR = Some(RefCell::new(coordinator.clone()));
	}

	let mut view = view::View::new(coordinator.clone());

	executor.queue(controller::run_main_controller());

	while executor.num_queued_tasks() > 0 {
		executor.poll();
		coordinator.run(&mut game_state.borrow_mut(), &mut view);
		view.render(&game_state.borrow());
	}
}

static mut COORDINATOR: Option<RefCell<task::Coordinator>> = None;

pub fn get_coordinator() -> std::cell::Ref<'static, task::Coordinator> {
	unsafe {
		COORDINATOR.as_ref()
			.expect("Coordinator not initialised!")
			.borrow()
	}
}


fn generate_game_state() -> GameState {
	let mut state = GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}