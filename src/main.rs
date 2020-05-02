#![feature(nll)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![deny(rust_2018_idioms, future_incompatible)]
#![allow(elided_lifetimes_in_paths)]

mod prelude;
mod task;
mod types;
mod gamestate;
mod map;
mod item;
mod room;
mod view;
mod controller;
mod enemy;

use prelude::*;
use gamestate::GameState;
use view::View;
use task::Coordinator;

fn main() {
	if std::env::args().find(|s| s == "--text").is_some() {
		run_with_view(view::TextView::new());
	} else {
		run_with_view(view::GfxView::new());
	}
}


fn run_with_view(mut view: impl View) {
	let mut executor = task::Executor::new();

	let gamestate = generate_gamestate();
	let gamestate = Rc::new(RefCell::new(gamestate));
	let mut coordinator = Coordinator::new(Rc::clone(&gamestate));

	unsafe {
		COORDINATOR = Some(RefCell::new(coordinator.clone()));
	}

	executor.queue(controller::run_main_controller());

	while executor.num_queued_tasks() > 0 && !view.should_quit() {
		// TODO: update gamestate and resume until a view event is being waited for
		executor.poll();
		coordinator.run(&mut gamestate.borrow_mut(), &mut view);

		// TODO: update view until executor has things to wake
		view.update(&gamestate.borrow());
	}
}


static mut COORDINATOR: Option<RefCell<Coordinator>> = None;

pub fn get_coordinator() -> std::cell::Ref<'static, Coordinator> {
	unsafe {
		COORDINATOR.as_ref()
			.expect("Coordinator not initialised!")
			.borrow()
	}
}


fn generate_gamestate() -> GameState {
	let mut state = GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}