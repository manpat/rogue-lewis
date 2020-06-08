// use crate::prelude::*;
use super::vertex::*;
use std::marker::PhantomData;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MeshID<V: Vertex>(pub(super) usize, pub(super) PhantomData<*const V>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct UntypedMeshID(pub(super) usize);

impl<V: Vertex> From<MeshID<V>> for UntypedMeshID {
	fn from(MeshID(o, _): MeshID<V>) -> UntypedMeshID {
		UntypedMeshID(o)
	}
}

pub(super) struct Mesh {
	pub(super) descriptor: Descriptor,
	pub(super) element_count: u32,
	pub(super) vbo: u32,
	pub(super) ebo: u32
}

impl Mesh {
	pub(super) fn new(descriptor: Descriptor) -> Mesh {
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

	pub(super) fn bind(&self) {
		unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
		}
	}
}
