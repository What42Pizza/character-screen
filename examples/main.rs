pub use anyhow::*;
pub use log::info;
pub use sdl3::{
	event::{Event, WindowEvent},
	keyboard::{KeyboardState, Keycode},
};
pub use std::{path::PathBuf, time::Instant};



pub struct ProgramData {
	pub should_quit: bool,
	pub fps_counter: simple_gpu::FpsCounter,

	pub screen_size: (u32, u32),
	pub aspect_ratio: f32,

	pub render_data: charmap_renderer::RenderData,
	pub charmap: charmap_renderer::Charmap,
}



fn main() -> Result<()> {
	// enable logging
	if std::env::var("RUST_LOG").is_err() {
		unsafe {
			// safety: this seems to only be unsafe if other threads might be reading/writing env vars, and that should not be possible yet since this is the start of the program
			std::env::set_var("RUST_LOG", "warn");
		}
	}
	env_logger::init();

	let assets_path = get_assets_path()?;

	// sdl
	let sdl = sdl3::init()?;
	let video = sdl.video()?;
	let mut window = video
		.window("Character Screen", 96 * 10, 45 * 12)
		.position_centered()
		.resizable()
		.hidden()
		.metal_view()
		.build()?;
	let window_size = window.size();
	let mut event_pump = sdl.event_pump()?;

	// basics
	let gpu_instance = simple_gpu::init(wgpu::Limits::defaults(), wgpu::MemoryHints::Performance)?;
	let (window, window_surface) = simple_gpu::get_window_surface_mut(
		&gpu_instance,
		&mut window,
		window_size,
		wgpu::PresentMode::AutoVsync,
	);
	let mut window_surface = window_surface?;

	// textures
	let char_atlas_tex =
		charmap_renderer::load_character_atlas(assets_path.join("atlas.png"), &gpu_instance)?;
	let color_lut = simple_gpu::load_texture_from_path(
		assets_path.join("color_lut.png"),
		wgpu::FilterMode::Nearest,
		1,
		&gpu_instance,
	)?;
	debug_assert_eq!(color_lut.settings.size, (16, 16));
	let char_atlas_tex_view = char_atlas_tex
		.wgpu_texture
		.create_view(&wgpu::TextureViewDescriptor::default());
	let color_lut_tex_view = color_lut
		.wgpu_texture
		.create_view(&wgpu::TextureViewDescriptor::default());

	let render_data = charmap_renderer::make_render_data(window_surface.wgpu_format, &gpu_instance);

	// character map
	let mut charmap = charmap_renderer::create_charmap(
		"Main Charmap",
		(96, 45),
		&char_atlas_tex_view,
		&color_lut_tex_view,
		&render_data,
		&gpu_instance,
	);
	place_text(
		"All Characters: ",
		(0, 0),
		(0, 1, 0),
		&mut charmap.buffer,
		charmap.size,
	);
	for i in 16..16 + 128 {
		charmap.buffer[i * 4] = i as u8;
		charmap.buffer[i * 4 + 1] = 4;
		charmap.buffer[i * 4 + 2] = 1;
	}
	place_text(
		"HELLO, WORLD!",
		(16, 16),
		(0, 1, 0),
		&mut charmap.buffer,
		charmap.size,
	);
	place_text(
		"The quick brown fox jumped over the lazy dog.",
		(16, 17),
		(0, 1, 0),
		&mut charmap.buffer,
		charmap.size,
	);
	place_border(15, 61, 15, 18, (0, 1, 0), &mut charmap.buffer, charmap.size);
	simple_gpu::sync_buffer(&mut charmap.buffer, &gpu_instance);

	// assemble program's data
	let mut data = ProgramData {
		should_quit: false,
		fps_counter: simple_gpu::FpsCounter::new(),

		screen_size: window_size,
		aspect_ratio: window_size.0 as f32 / window_size.1 as f32,

		render_data,
		charmap,
	};



	window.show();
	while !data.should_quit {
		// handle events
		for event in event_pump.poll_iter() {
			match event {
				Event::Window {
					window_id,
					win_event:
						WindowEvent::PixelSizeChanged(new_width, new_height)
						| WindowEvent::Resized(new_width, new_height),
					..
				} if window_id == window.id() => {
					simple_gpu::reconfigure_window_surface(
						&mut window_surface,
						&gpu_instance,
						(new_width as u32, new_height as u32),
					);
					data.aspect_ratio = new_width as f32 / new_height as f32;
				}
				Event::Quit { .. }
				| Event::Window {
					win_event: WindowEvent::CloseRequested,
					..
				}
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => {
					println!("closing"); // note: this can get both a quit event and a close requested event in the same frame
					data.should_quit = true;
				}
				e => {
					info!("Unknown event: {e:?}");
				}
			}
		}

		// render
		let (surface_tex, surface_tex_view, mut command_encoder) =
			match simple_gpu::start_frame("render frame", &window_surface, &gpu_instance) {
				simple_gpu::StartFrameResult::Some(tex, view, commands) => (tex, view, commands),
				simple_gpu::StartFrameResult::None => continue,
				simple_gpu::StartFrameResult::Error => {
					simple_gpu::reconfigure_window_surface(
						&mut window_surface,
						&gpu_instance,
						window.size(),
					);
					continue;
				}
			};

		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Character Map Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &surface_tex_view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Load,
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});

		charmap_renderer::render(&[&data.charmap], &mut render_pass, &mut data.render_data);

		simple_gpu::finish_render_pass(render_pass);

		simple_gpu::finish_frame(command_encoder, surface_tex, &gpu_instance);

		data.fps_counter.tick();
	}

	Ok(())
}



pub fn place_text(
	text: impl AsRef<str>,
	pos: (u16, u16),
	colors: (u8, u8, u8),
	charmap_buffer: &mut simple_gpu::GpuBuffer<u8>,
	charmap_size: (u16, u16),
) {
	let mut pos = (pos.0 as usize + pos.1 as usize * charmap_size.0 as usize) * 4;
	for c in text.as_ref().chars() {
		charmap_buffer[pos] = c as u8;
		pos += 1;
		charmap_buffer[pos] = colors.0;
		pos += 1;
		charmap_buffer[pos] = colors.1;
		pos += 1;
		charmap_buffer[pos] = colors.2;
		pos += 1;
	}
}

pub fn place_border(
	left_edge: u16,
	right_edge: u16,
	top_edge: u16,
	bottom_edge: u16,
	colors: (u8, u8, u8),
	charmap_buffer: &mut simple_gpu::GpuBuffer<u8>,
	charmap_size: (u16, u16),
) {
	macro_rules! place_char {
		($char:expr, $x:ident, $y:ident) => {
			charmap_buffer[($x + $y * charmap_size.0) as usize * 4] = $char;
			charmap_buffer[($x + $y * charmap_size.0) as usize * 4 + 1] = colors.0;
			charmap_buffer[($x + $y * charmap_size.0) as usize * 4 + 2] = colors.1;
			charmap_buffer[($x + $y * charmap_size.0) as usize * 4 + 3] = colors.2;
		};
	}
	for x in left_edge + 1..=right_edge - 1 {
		place_char!(2, x, top_edge);
		place_char!(2, x, bottom_edge);
	}
	for y in top_edge + 1..=bottom_edge - 1 {
		place_char!(3, left_edge, y);
		place_char!(3, right_edge, y);
	}
	place_char!(1, left_edge, top_edge);
	place_char!(1, right_edge, top_edge);
	place_char!(1, left_edge, bottom_edge);
	place_char!(1, right_edge, bottom_edge);
}



pub fn get_assets_path() -> Result<PathBuf> {
	let mut output = std::env::current_exe()?;
	loop {
		output.push("assets");
		if output.exists() {
			return Ok(output);
		}
		output.pop();
		if !output.pop() {
			bail!("Failed to find assets folder near executable or any parent folders.");
		}
	}
}
