
use std::marker::PhantomData;

use crate::prelude::*;

use super::mesh_builder::MeshBuilder;
use super::shader::*;
use super::vertex::*;
use super::mesh::*;


pub struct Core {
	shaders: Vec<Shader>,
	meshes: Vec<Mesh>,

	bound_shader: Option<ShaderID>,
	bound_mesh: Option<UntypedMeshID>,
}


impl Core {
	pub fn new() -> Core {
		Core {
			shaders: Vec::new(),
			meshes: Vec::new(),

			bound_shader: None,
			bound_mesh: None,
		}
	}

	pub fn set_bg_color(&mut self, c: Color) {
		unsafe {
			let (r,g,b,a) = c.to_tuple();
			gl::ClearColor(r, g, b, a);
		}
	}

	pub fn set_viewport(&mut self, size: Vec2i) {
		unsafe {
			let Vec2i{x, y} = size;
			gl::Viewport(0, 0, x, y);
		}
	}

	pub fn clear(&mut self) {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT|gl::STENCIL_BUFFER_BIT);
		}
	}

	// Shaders
	pub fn new_shader(&mut self, vsrc: &str, fsrc: &str, attribs: &[&str]) -> ShaderID {
		self.shaders.push(Shader::new(vsrc, fsrc, attribs));
		ShaderID(self.shaders.len()-1)
	}

	pub fn use_shader(&mut self, id: ShaderID) {
		unsafe {
			let shader = self.shaders.get(id.0).expect("Tried to use invalid shader");
			gl::UseProgram(shader.handle);
			self.bound_shader = Some(id);


			for i in 0..shader.attribute_count {
				gl::EnableVertexAttribArray(i);
			}

			for i in shader.attribute_count..8 {
				gl::DisableVertexAttribArray(i);
			}
		}
	}

	pub fn set_uniform_mat4(&mut self, name: &str, value: &Mat4) {
		unsafe {
			let shader_id = self.bound_shader.unwrap();
			let shader = &self.shaders[shader_id.0];
			let name = std::ffi::CString::new(name.as_bytes()).unwrap();

			let loc = gl::GetUniformLocation(shader.handle, name.as_ptr());
			gl::UniformMatrix4fv(loc, 1, 0, &value.transpose() as *const _ as *const f32);
		}
	}

	// Meshes
	pub fn new_mesh<V: Vertex>(&mut self) -> MeshID<V> {
		self.meshes.push(Mesh::new(V::descriptor()));
		MeshID(self.meshes.len()-1, PhantomData)
	}

	fn bind_mesh<V: Vertex>(&mut self, id: MeshID<V>) {
		let untyped_id = UntypedMeshID::from(id);
		if self.bound_mesh == Some(untyped_id) {
			return;
		}

		let mesh = self.meshes.get(id.0).expect("Tried to bind invalid mesh");
		mesh.bind();
		self.bound_mesh = Some(untyped_id);
	}

	pub fn update_mesh<V: Vertex>(&mut self, id: MeshID<V>, vs: &[V], es: &[u16]) {
		self.bind_mesh(id);

		let mesh = self.meshes.get_mut(id.0).expect("Tried to bind invalid mesh");
		mesh.element_count = es.len() as _;

		unsafe {
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vs.len() * std::mem::size_of::<V>()) as _,
				vs.as_ptr() as *const _,
				gl::STATIC_DRAW
			);

			gl::BufferData(
				gl::ELEMENT_ARRAY_BUFFER,
				(es.len() * std::mem::size_of::<u16>()) as _,
				es.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
	}

	pub fn update_mesh_from<V: Vertex>(&mut self, mb: &MeshBuilder<V>) {
		self.update_mesh(mb.mesh_id, &mb.vs, &mb.es);
	}

	pub fn draw_mesh<V: Vertex>(&mut self, id: MeshID<V>) {
		self.bind_mesh(id);

		let mesh = self.meshes.get(id.0).expect("Tried to bind invalid mesh");
		mesh.descriptor.bind();

		unsafe {
			gl::DrawElements(
				gl::TRIANGLES,
				mesh.element_count as _,
				gl::UNSIGNED_SHORT,
				std::ptr::null()
			);
		}
	}
}