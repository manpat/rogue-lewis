pub use rand::{Rng, random, seq::SliceRandom, seq::IteratorRandom};

pub use std::rc::Rc;
pub use std::cell::RefCell;
pub use std::future::Future;

pub use crate::types::*;
pub use crate::get_coordinator;

// pub type Result<T> = std::result::Result<T, failure::Error>;

pub fn rng() -> rand::rngs::ThreadRng { rand::thread_rng() }


pub fn choose_with_weights<T: Copy>(values: &[T], weights: &[i32]) -> T {
	use rand::distributions::weighted::WeightedIndex;
	use rand::distributions::Distribution;

	assert!(values.len() == weights.len());

	let dist = WeightedIndex::new(weights).unwrap();
	values[dist.sample(&mut rng())]
}

pub type GameStateHandle = Rc<RefCell<crate::game_state::GameState>>;
