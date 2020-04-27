use std::error::Error;
use sdl2::Sdl;
use sdl2::video::{GLContext, GLProfile};

pub struct Window {
	#[allow(dead_code)]
	sdl: Sdl,

	window: sdl2::video::Window,

	#[allow(dead_code)]
	gl_ctx: GLContext,
}


impl Window {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let sdl = sdl2::init()?;
		let video = sdl.video()?;

		let attr = video.gl_attr();
		attr.set_context_profile(GLProfile::Core);
		attr.set_context_version(3, 2);
		attr.set_context_flags().debug().set();

		let window = video
			.window("rogue-lewis", 640, 640)
			.opengl()
			.build()?;

		let gl_ctx = window.gl_create_context()?;

		gl::load_with(|s| video.gl_get_proc_address(s) as *const _);
		unsafe {
			let mut vao = 0;
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			
			gl::PointSize(2.0);

			gl::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());
			gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

			// Disable performance messages
			gl::DebugMessageControl(
				gl::DONT_CARE,
				gl::DEBUG_TYPE_PERFORMANCE,
				gl::DONT_CARE,
				0, std::ptr::null(),
				0 // false
			);

			// Disable notification messages
			gl::DebugMessageControl(
				gl::DONT_CARE,
				gl::DONT_CARE,
				gl::DEBUG_SEVERITY_NOTIFICATION,
				0, std::ptr::null(),
				0 // false
			);
		}

		println!("{:?}", video.gl_attr().context_version());

		Ok(Window {
			sdl,
			window,
			gl_ctx,
		})
	}

	pub fn event_pump(&mut self) -> sdl2::EventPump {
		self.sdl.event_pump().unwrap()
	}

	pub fn swap(&mut self) {
		self.window.gl_swap_window();
	}
}


extern "system" fn gl_message_callback(source: u32, ty: u32, id: u32, severity: u32,
	length: i32, msg: *const i8, ud: *mut std::ffi::c_void)
{
	let severity = match severity {
		gl::DEBUG_SEVERITY_LOW => "low",
		gl::DEBUG_SEVERITY_MEDIUM => "medium",
		gl::DEBUG_SEVERITY_HIGH => "high",
		gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
		_ => panic!("Unknown severity {}", severity),
	};

	let ty = match ty {
		gl::DEBUG_TYPE_ERROR => "error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behaviour",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
		gl::DEBUG_TYPE_PORTABILITY => "portability",
		gl::DEBUG_TYPE_PERFORMANCE => "performance",
		gl::DEBUG_TYPE_OTHER => "other",
		_ => panic!("Unknown type {}", ty),
	};

	let source = match source {
		gl::DEBUG_SOURCE_API => "api",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
		gl::DEBUG_SOURCE_APPLICATION => "application",
		gl::DEBUG_SOURCE_OTHER => "other",
		_ => panic!("Unknown source {}", source),
	};

	eprintln!("GL ERROR!");
	eprintln!("Source:   {}", source);
	eprintln!("Severity: {}", severity);
	eprintln!("Type:     {}", ty);

	unsafe {
		let msg = std::ffi::CStr::from_ptr(msg as _).to_str().unwrap();
		eprintln!("Message: {}", msg);
	}

	panic!("GL ERROR!");
}