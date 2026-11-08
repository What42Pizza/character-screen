#![allow(unused)]
#![warn(unused_must_use)]



pub const CHAR_WIDTH: usize = 10;
pub const CHAR_HEIGHT: usize = 12;



pub use anyhow::*;
pub use bytemuck::Zeroable;
pub use log::info;
pub use sdl3::{
	event::{Event, WindowEvent},
	keyboard::{KeyboardState, Keycode},
};
pub use std::{path::PathBuf, time::Instant};



pub mod load_textures;
pub use load_textures::*;
pub mod render;
pub use render::*;
pub mod wgpu_defs;
pub use wgpu_defs::*;



pub struct ProgramData {
	
	pub should_quit: bool,
	pub last_dt_instant: Instant,
	pub fps_counter: simple_gpu::FpsCounter,
	
	pub screen_size: (u32, u32),
	pub aspect_ratio: f32,
	
	pub render_data: RenderData,
	pub char_map_buffer: simple_gpu::GpuBuffer<u8>,
	pub char_map_bind_group: wgpu::BindGroup,
	
}

pub struct RenderData {
	
	pub char_atlas_tex: wgpu::Texture,
	pub char_atlas_view: wgpu::TextureView,
	pub main_shader: simple_gpu::Shaders,
	
	pub uniforms_buffer: simple_gpu::UniformsBuffer<UniformsRawData>,
	pub nearest_sampler: wgpu::Sampler,
	pub main_bind_group_layout: wgpu::BindGroupLayout,
	pub char_map_bind_group_layout: wgpu::BindGroupLayout,
	pub main_bind_group: wgpu::BindGroup,
	pub pipeline_layout: wgpu::PipelineLayout,
	pub pipeline: wgpu::RenderPipeline,
	pub curr_pipeline_output_format: wgpu::TextureFormat,
	
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct UniformsRawData {
	pub charmap_size: [u32; 2],
	pub target_size: [u32; 2],
}

pub fn update_uniforms(uniforms_buffer: &mut simple_gpu::UniformsBuffer<UniformsRawData>, target_size: (u32, u32), charmap_size: (u16, u16)) {
	let uniforms = &mut uniforms_buffer.cpu_copy;
	uniforms.charmap_size = [charmap_size.0 as u32, charmap_size.1 as u32];
	uniforms.target_size = target_size.into();
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
		.window("Character Screen", 1280, 720)
		.position_centered()
		.resizable()
		.hidden()
		.metal_view()
		.build()?;
	let window_size = window.size();
	let mut event_pump = sdl.event_pump()?;
	
	// basics
	let mut gpu_instance = simple_gpu::init(wgpu::Limits::defaults(), wgpu::MemoryHints::Performance)?;
	let (window, window_surface) = simple_gpu::get_window_surface_mut(
		&gpu_instance,
		&mut window,
		window_size,
		wgpu::PresentMode::AutoVsync,
	);
	let mut window_surface = window_surface?;
	
	// shaders
	let shaders_path = assets_path.join("shaders");
	let main_shader = simple_gpu::make_glsl_shaders(
		"shaders/main.vsh",
		include_str!("shaders/main.vsh"),
		"shaders/main.fsh",
		include_str!("shaders/main.fsh"),
		&gpu_instance,
		&[],
	);
	
	// textures
	let textures_path = assets_path.join("textures");
	let char_atlas_tex = load_all_textures(textures_path, &gpu_instance)?;
	
	let render_data = make_render_data(char_atlas_tex, main_shader, window_surface.wgpu_format, &gpu_instance);
	
	// character map
	let (char_map_buffer, char_map_bind_group, size) = make_character_map((96, 45), &render_data, &gpu_instance);
	
	// assemble program's data
	let mut data = ProgramData {
		
		should_quit: false,
		last_dt_instant: Instant::now(),
		fps_counter: simple_gpu::FpsCounter::new(),
		
		screen_size: window_size,
		aspect_ratio: window_size.0 as f32 / window_size.1 as f32,
		
		render_data,
		char_map_buffer,
		char_map_bind_group,
		
	};
	
	
	
	window.show();
	while !data.should_quit {
		// initial update
		let new_dt_instant = Instant::now();
		let dt = new_dt_instant
			.duration_since(data.last_dt_instant)
			.as_secs_f32();
		data.last_dt_instant = new_dt_instant;
		
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
		
		// update
		let keyboard_state = KeyboardState::new(&event_pump);
		
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
			color_attachments: &[
				Some(wgpu::RenderPassColorAttachment {
					view: &surface_tex_view,
					depth_slice: None,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Load,
						store: wgpu::StoreOp::Store,
					},
				}),
			],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});
		
		//prepare_rendering(target_size, &mut render_pass, &mut data.render_data, &gpu_instance);
		render(&[(&data.char_map_bind_group, size)], data.screen_size, &mut render_pass, &mut data.render_data, &gpu_instance);
		
		simple_gpu::finish_render_pass(render_pass);
		
		simple_gpu::finish_frame(command_encoder, surface_tex, &gpu_instance);
		
		data.fps_counter.tick();
	}
	
	Ok(())
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
