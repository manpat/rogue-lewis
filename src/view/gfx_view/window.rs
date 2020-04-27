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

		let window = video
			.window("rogue-lewis", 640, 420)
			.opengl()
			.build()?;

		let gl_ctx = window.gl_create_context()?;

		gl::load_with(|s| video.gl_get_proc_address(s) as *const _);
		let mut vao = 0;
		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
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