use crate::prelude::*;
use std::error::Error;

pub struct Window {
	context: glutin::WindowedContext<glutin::PossiblyCurrent>,
	events_loop: glutin::EventsLoop,
}


impl Window {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let events_loop = glutin::EventsLoop::new();

		let window = glutin::WindowBuilder::new()
			.with_title("rogue-lewis")
			.with_resizable(true);

		let context = glutin::ContextBuilder::new()
			.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
			.with_gl_profile(glutin::GlProfile::Core)
			.with_gl_debug_flag(true)
			.build_windowed(window, &events_loop)?;

		let context = unsafe { context.make_current().unwrap() };

		gl::load_with(|s| context.get_proc_address(s) as *const _);

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

		Ok(Window {
			context,
			events_loop,
		})
	}

	pub fn size(&self) -> Vec2i {
		let (x, y): (u32, u32) = self.context.window()
			.get_inner_size()
			.unwrap()
			.to_physical(self.dpi())
			.into();

		Vec2i::new(x as i32, y as i32)
	}

	pub fn dpi(&self) -> f64 {
		self.context.window().get_hidpi_factor()
	} 

	pub fn poll_events(&mut self) -> Vec<glutin::WindowEvent> {
		let mut events = Vec::new();

		self.events_loop.poll_events(|event| {
			if let glutin::Event::WindowEvent{event, ..} = event {
				events.push(event);
			}
		});

		events
	}

	pub fn swap(&mut self) {
		self.context.swap_buffers().unwrap();
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