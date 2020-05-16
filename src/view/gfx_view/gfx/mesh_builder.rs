use super::vertex::Vertex;
use super::Gfx;
use super::mesh::MeshID;

pub struct MeshBuilder<V: Vertex> {
	pub mesh_id: MeshID<V>,
	pub vs: Vec<V>,
	pub es: Vec<u16>,
}

impl<V: Vertex> MeshBuilder<V> {
	pub fn new(mesh_id: MeshID<V>) -> MeshBuilder<V> {
		MeshBuilder {
			mesh_id,
			vs: Vec::new(),
			es: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.vs.clear();
		self.es.clear();
	}

	pub fn add_geometry<I, Item>(&mut self, verts: &[V], indices: I) where I: IntoIterator<Item=Item>, Item: IntoIndex {
		let start = self.vs.len();
		if start >= 0xffff {
			panic!("Too many verts!");
		}

		self.vs.extend_from_slice(verts);
		self.es.extend(indices.into_iter().map(|i| i.into_index() + start as u16));
	}

	pub fn add_quad(&mut self, verts: &[V]) {
		self.add_geometry(verts, &[0, 1, 2, 0, 2, 3]);
	}

	pub fn add_tri_fan(&mut self, vs: &[V]) {
		assert!(vs.len() >= 3);

		let indices = (1..vs.len()-1)
			.flat_map(|i| {
				let i = i as u16;
				let is = [0, i, i+1];
				(0..3).map(move |i| is[i])
			});

		self.add_geometry(vs, indices);
	}

	pub fn add_tri_strip(&mut self, vs: &[V]) {
		assert!(vs.len() >= 3);

		let indices = (0..vs.len()-2)
			.flat_map(|i| (0..3).map(move |offset| i as u16 + offset));

		self.add_geometry(vs, indices);
	}
}



pub trait IntoIndex {
	fn into_index(self) -> u16;
}

impl IntoIndex for u16 {
	fn into_index(self) -> u16 { self }
}

impl<'a> IntoIndex for &'a u16 {
	fn into_index(self) -> u16 { *self }
}