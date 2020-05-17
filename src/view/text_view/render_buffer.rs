use crate::prelude::*;
use std::fmt;


#[derive(Debug, Clone)]
pub struct RenderBuffer {
	buffer: Vec<char>,
	width: usize,
	height: usize,
}



impl RenderBuffer {
	pub fn new(width: usize, height: usize) -> RenderBuffer {
		RenderBuffer {
			buffer: vec![' '; width * height],
			width, height,
		}
	}

	pub fn fill(&mut self, ch: char) {
		for dst in self.buffer.iter_mut() {
			*dst = ch;
		}
	}

	pub fn write(&mut self, Location(x, y): Location, ch: char) {
		assert!(x >= 0 && x < self.width as i32);
		assert!(y >= 0 && y < self.height as i32);

		let index = x as usize + (self.height - y as usize - 1) * self.width;
		self.buffer[index] = ch;
	}
}

impl fmt::Display for RenderBuffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let get_ch = |x, y| {
			let index = x + y * self.width;
			let ch = self.buffer[index];

			if ch == '\0' {
				' '
			} else {
				ch
			}
		};

		for y in 0..self.height-1 {
			for x in 0..self.width {
				write!(f, "{}", get_ch(x, y))?;
			}
			writeln!(f, "")?;
		}

		for x in 0..self.width {
			write!(f, "{}", get_ch(x, self.height-1))?;
		}

		Ok(())
	}
}

