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
use task::Executor;

fn main() {
	if std::env::args().find(|s| s == "--text").is_some() {
		run_with_view(view::TextView::new());
	} else {
		run_with_view(view::GfxView::new());
	}
}


fn run_with_view(mut view: impl View) {
	let gamestate = generate_gamestate();
	let gamestate = Rc::new(RefCell::new(gamestate));

	unsafe {
		let executor = task::Executor::new(Rc::clone(&gamestate));
		EXECUTOR = Some(executor);
	}

	get_executor().queue(controller::run_main_controller());

	view.init(&gamestate.borrow());

	while get_executor().num_queued_tasks() > 0 && !view.should_quit() {
		get_executor().resume_tasks();
		get_executor().process_commands(&mut gamestate.borrow_mut(), &mut view);
		view.update(&gamestate.borrow());
	}
}


static mut EXECUTOR: Option<Executor> = None;

pub fn get_executor() -> &'static Executor {
	unsafe {
		EXECUTOR.as_ref()
			.expect("Executor not initialised!")
	}
}


fn generate_gamestate() -> GameState {
	let mut state = GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}