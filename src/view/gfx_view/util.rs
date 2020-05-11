use crate::prelude::*;




pub fn location_to_world(Location(x, y): Location) -> Vec2 {
	Vec2i::new(x, -y).to_vec2()*2.0
}