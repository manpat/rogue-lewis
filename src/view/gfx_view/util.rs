use crate::prelude::*;




pub fn location_to_world(Location(x, y): Location) -> Vec2 {
	Vec2i::new(x, -y).to_vec2()*2.0
}




pub fn intersect_plane(plane_point: Vec3, plane_normal: Vec3, line_point: Vec3, line_direction: Vec3) -> Option<Vec3> {
	let line_direction = line_direction.normalize();

	if plane_normal.dot(line_direction).abs() < 0.01 {
		return None;
	}

	let t = (plane_normal.dot(plane_point) - plane_normal.dot(line_point)) / plane_normal.dot(line_direction);
	Some(line_point + line_direction * t)
}


// pub fn intersect_ground(line_point: Vec3, line_direction: Vec3) -> Vec3 {
// 	let plane_point = Vec3::zero();
// 	let plane_normal = Vec3::from_y(1.0);

// 	intersect_plane(plane_point, plane_normal, line_point, line_direction)
// 		.expect("Camera forward perpendicular to ground plane")
// }