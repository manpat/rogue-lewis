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

	let state = generate_game_state();
	let mut state = Rc::new(RefCell::new(state));

	executor.queue(controller::run_main_controller(Rc::clone(&state)));

	while executor.num_queued_tasks() > 0 {
		executor.poll();

		// let ctl_ctx = &mut ControllerContext::new(&mut state, &mut executor);

		// match controllers.last_mut().unwrap().run_command(ctl_ctx, command[0]) {
		// 	None => {}

		// 	Some(Event::Enter(mut new)) => {
		// 		controllers.last_mut().unwrap().leave(ctl_ctx);

		// 		new.enter(ctl_ctx);
		// 		controllers.push(new);
		// 	}

		// 	Some(Event::Leave) => {
		// 		if let Some(mut prev) = controllers.pop() {
		// 			prev.leave(ctl_ctx);
		// 		}

		// 		controllers.last_mut().unwrap().enter(ctl_ctx);
		// 	}

		// 	Some(Event::Restart) => {
		// 		println!("The walls warp and shift around you and your sense of reality temporarily disolves");
		// 		println!("You are unsure if any of the events you've experienced until now actually happened");
		// 		*ctl_ctx.state = generate_game_state();
		// 		let mut main_controller = Controller::from(MainController);
		// 		main_controller.enter(ctl_ctx);
		// 		controllers = vec![main_controller];
		// 	}

		// 	Some(Event::Win) => {
		// 		println!("You win!");
		// 		break
		// 	}
		// 	Some(Event::Lose) => {
		// 		println!("You lost");
		// 		break
		// 	}
		// 	Some(Event::Quit) => {
		// 		println!("See ya!");
		// 		break
		// 	}
		// }
	}
}


fn read_command() -> String {
	use std::io::{Write, BufRead};

	print!("> ");

	std::io::stdout().flush()
		.expect("Failed to flush");

	std::io::stdin().lock()
		.lines().next()
		.expect("EOF")
		.expect("Failed to read stdin")
		.to_ascii_lowercase()
}

fn generate_game_state() -> game_state::GameState {
	let mut state = game_state::GameState::new();

	let mut map_builder = map::MapBuilder::new(&mut state.map);
	map_builder.generate_random_walk();

	state
}