pub mod executor;

pub use executor::Executor;



pub async fn get_player_command() -> String {
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
			break command
		}
	}
} 
