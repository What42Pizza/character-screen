#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::large_enum_variant)]
#![allow(
	clippy::cast_possible_truncation,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::cast_possible_wrap
)]
#![feature(impl_trait_in_bindings)]

//! Character Screen
//!
//! This is a crate for rendering character maps.
//!
//! - Each cell in the character map defines a character to render and three colors to render with.
//! - Colors in the character map cells are indices into a color lut texture
//! - Each character texture uses the red channel to mix the first two colors, the green channel for mixing the first mix result with the third color, and the alpha channel for opacity



/// This is the width of characters. It should be customizable but idk how to do that well
pub const CHAR_WIDTH: usize = 10;
/// This is the height of characters. It should be customizable but idk how to do that well
pub const CHAR_HEIGHT: usize = 14;



/// Functions for initializing wgpu stuff
pub mod wgpu_defs;
pub use wgpu_defs::*;
/// Functions dealing with character texture atlases
pub mod atlas;
pub use atlas::*;



use std::ops::{Deref, DerefMut};



/// Holds all the miscellaneous data for rendering character maps
pub struct CharmapRenderData {
	/// A simple non-filtering sampler
	pub nearest_sampler: wgpu::Sampler,
	/// The shader used for rendering the character map
	pub main_shader: simple_gpu::Shaders,
	/// The bind group layout that the shader expects
	pub bind_group_layout: wgpu::BindGroupLayout,
	/// The wgpu pipeline layout (used for all charmap renders)
	pub pipeline_layout: wgpu::PipelineLayout,
	/// The wgpu pipeline (used for all charmap renders)
	pub pipeline: wgpu::RenderPipeline,
	/// The output format of the current wgpu pipeline
	pub curr_pipeline_output_format: wgpu::TextureFormat,
}



/// Misc data used for rendering charmaps
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CharmapRawUniforms {
	/// size of the current charmap
	pub charmap_size: [u32; 2],
}



/// Holds all the data for a character map
///
/// The character data is stored is `Self.buffer.cpu_copy`, which this type can be automatically dereferenced as
pub struct Charmap {
	/// Holds the name, this is given to wgpu for easier debugging
	pub name: String,
	/// The size that this charmap was created as
	pub size: (u16, u16),
	/// Holds the size of the charmap
	pub uniforms_buffer: simple_gpu::UniformsBuffer<CharmapRawUniforms>,
	/// Holds both the cpu copy and (a handle for) the gpu copy of the charmap data
	pub buffer: simple_gpu::GpuBuffer<u8>,
	/// Holds the resources that are given to the shader
	pub bind_group: wgpu::BindGroup,
}

impl Charmap {
	/// Places a character (or generic value) at a position within the charmap
	#[inline]
	pub fn place(&mut self, value: u8, pos: (u16, u16), colors: (u8, u8, u8)) {
		let i = (pos.0 as usize + pos.1 as usize * self.size.0 as usize) * 4;
		self.buffer[i] = value;
		self.buffer[i + 1] = colors.0;
		self.buffer[i + 2] = colors.1;
		self.buffer[i + 3] = colors.2;
	}
	/// Gets the character (or generic value) at a position within the charmap
	#[must_use]
	#[inline]
	pub fn get(&self, pos: (u16, u16)) -> (u8, u8, u8, u8) {
		let i = (pos.0 as usize + pos.1 as usize * self.size.0 as usize) * 4;
		(
			self.buffer[i],
			self.buffer[i + 1],
			self.buffer[i + 2],
			self.buffer[i + 3],
		)
	}
	/// Sends the data in `Self.buffer` to the gpu
	#[inline]
	pub fn update_gpu(&mut self, gpu_instance: &simple_gpu::GpuInstance) {
		simple_gpu::sync_buffer(&mut self.buffer, gpu_instance);
	}
}

impl Deref for Charmap {
	type Target = Vec<u8>;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.buffer
	}
}

impl DerefMut for Charmap {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.buffer
	}
}



/// Does the actual rendering
#[inline]
pub fn render(
	charmaps: &[&Charmap],
	render_pass: &mut wgpu::RenderPass,
	render_data: &mut CharmapRenderData,
) {
	render_pass.set_pipeline(&render_data.pipeline);
	for charmap in charmaps {
		render_pass.set_bind_group(0, &charmap.bind_group, &[]);
		render_pass.draw(0..3, 0..1);
	}
}
