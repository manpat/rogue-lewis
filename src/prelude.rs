pub use crate::types::*;
pub use rand::{Rng, random, seq::SliceRandom, seq::IteratorRandom};

pub type Result<T> = std::result::Result<T, failure::Error>;


pub fn rng() -> rand::rngs::ThreadRng { rand::thread_rng() }

pub fn read_line() -> String {
	use std::io::BufRead;
	std::io::stdin().lock()
		.lines().next()
		.expect("EOF")
		.expect("Failed to read stdin")
}
