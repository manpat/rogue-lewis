use crate::prelude::*;
use super::vertex::*;
use super::mesh_builder::MeshBuilder;
use std::marker::PhantomData;

pub struct Gfx {
	shaders: Vec<Shader>,
	meshes: Vec<Mesh>,

	bound_shader: Option<ShaderID>,
	bound_mesh: Option<UntypedMeshID>,
}


impl Gfx {
	pub fn new() -> Gfx {
		Gfx {
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


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShaderID(usize);

struct Shader {
	handle: u32,
	attribute_count: u32,
}

impl Shader {
	fn new(vsrc: &str, fsrc: &str, attribs: &[&str]) -> Shader {
		let vsh = compile_shader(vsrc, gl::VERTEX_SHADER);
		let fsh = compile_shader(fsrc, gl::FRAGMENT_SHADER);

		let program = link_shader(vsh, fsh, attribs);

		Shader {
			handle: program,
			attribute_count: attribs.len() as _,
		}
	}
}

fn compile_shader(src: &str, ty: u32) -> u32 {
	use std::ffi::CString;
	use std::str;

	unsafe {
		let handle = gl::CreateShader(ty);
		let src = CString::new(src.as_bytes()).unwrap();

		gl::ShaderSource(handle, 1, &src.as_ptr(), std::ptr::null());
		gl::CompileShader(handle);

		let mut status = 0;
		gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);

		if status == 0 {
			let mut length = 0;
			gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut length);

			let mut buffer = vec![0u8; length as usize];
			gl::GetShaderInfoLog(
				handle,
				length,
				std::ptr::null_mut(),
				buffer.as_mut_ptr() as *mut _
			);

			let error = str::from_utf8(&buffer[..buffer.len()-1]).unwrap();

			panic!("Shader compile failed!\n{}", error);
		}

		handle
	}
}

fn link_shader(vsh: u32, fsh: u32, attribs: &[&str]) -> u32 {
	use std::ffi::CString;

	unsafe {
		let handle = gl::CreateProgram();
		gl::AttachShader(handle, vsh);
		gl::AttachShader(handle, fsh);

		for (i, &a) in attribs.iter().enumerate() {
			let a = CString::new(a.as_bytes()).unwrap();
			gl::BindAttribLocation(handle, i as u32, a.as_ptr());
		}

		gl::LinkProgram(handle);

		let mut status = 0;
		gl::GetProgramiv(handle, gl::LINK_STATUS, &mut status);

		if status == 0 {
			let mut buf = [0u8; 1024];
			let mut len = 0;
			gl::GetProgramInfoLog(handle, buf.len() as _, &mut len, buf.as_mut_ptr() as _);

			panic!("shader link failed: {}", std::str::from_utf8(&buf[..len as usize]).unwrap());
		}

		gl::DeleteShader(vsh);
		gl::DeleteShader(fsh);

		handle
	}
}



#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MeshID<V: Vertex>(usize, PhantomData<*const V>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct UntypedMeshID(usize);

impl<V: Vertex> From<MeshID<V>> for UntypedMeshID {
	fn from(MeshID(o, _): MeshID<V>) -> UntypedMeshID {
		UntypedMeshID(o)
	}
}

struct Mesh {
	descriptor: Descriptor,
	element_count: u32,
	vbo: u32, ebo: u32
}

impl Mesh {
	fn new(descriptor: Descriptor) -> Mesh {
		unsafe {
			let mut buffers = [0; 2];
			gl::GenBuffers(2, buffers.as_mut_ptr());

			let [vbo, ebo] = buffers;
			Mesh {
				descriptor,
				element_count: 0,
				vbo, ebo
			}
		}
	}

	fn bind(&self) {
		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
		}
	}
}
