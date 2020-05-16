


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShaderID(pub(super) usize);

pub(super) struct Shader {
	pub(super) handle: u32,
	pub(super) attribute_count: u32,
}

impl Shader {
	pub(super) fn new(vsrc: &str, fsrc: &str, attribs: &[&str]) -> Shader {
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
