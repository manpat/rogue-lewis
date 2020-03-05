pub use crate::types::*;
pub use rand::{Rng, random, seq::SliceRandom, seq::IteratorRandom};

// pub type Result<T> = std::result::Result<T, failure::Error>;

pub fn rng() -> rand::rngs::ThreadRng { rand::thread_rng() }
