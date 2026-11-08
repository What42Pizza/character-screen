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



/// Everything this crate needs for rendering character maps
pub struct RenderData {
	/// The wgpu texture for the character textures, must hold 16 chars x 16 chars (160x224 total size)
	pub char_atlas_tex: wgpu::Texture,
	/// The wgpu tex view for the character textures, must hold 16 chars x 16 chars (160x224 total size)
	pub char_atlas_view: wgpu::TextureView,
	/// The wgpu texture for the color lut, must be 16x16
	pub color_lut_tex: wgpu::Texture,
	/// The wgpu tex view for the color lut, must be 16x16
	pub color_lut_view: wgpu::TextureView,
	/// The shader used for rendering the character map
	pub main_shader: simple_gpu::Shaders,

	/// Uniforms for misc data
	pub uniforms_buffer: simple_gpu::UniformsBuffer<UniformsRawData>,
	/// A simple non-filtering sampler
	pub nearest_sampler: wgpu::Sampler,
	/// The main wgpu bind group layout (used for all charmap renders)
	pub main_bind_group_layout: wgpu::BindGroupLayout,
	/// The charmap wgpu bind group layout (used for specific charmap renders)
	pub charmap_bind_group_layout: wgpu::BindGroupLayout,
	/// The main wgpu bind group (used for all charmap renders)
	pub main_bind_group: wgpu::BindGroup,
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
pub struct UniformsRawData {
	/// size of the current charmap
	pub charmap_size: [u32; 2],
}

/// Updates the uniforms buffer
#[inline]
pub fn update_uniforms(
	uniforms_buffer: &mut simple_gpu::UniformsBuffer<UniformsRawData>,
	charmap_size: (u16, u16),
) {
	let uniforms = &mut uniforms_buffer.cpu_copy;
	uniforms.charmap_size = [u32::from(charmap_size.0), u32::from(charmap_size.1)];
}



/// Does the actual rendering
#[inline]
pub fn render(
	charmaps: &[(&wgpu::BindGroup, (u16, u16))],
	render_pass: &mut wgpu::RenderPass,
	render_data: &mut RenderData,
	gpu_instance: &simple_gpu::GpuInstance,
) {
	render_pass.set_pipeline(&render_data.pipeline);
	render_pass.set_bind_group(0, &render_data.main_bind_group, &[]);
	for (charmap_bind, charmap_size) in charmaps {
		update_uniforms(&mut render_data.uniforms_buffer, *charmap_size);
		simple_gpu::sync_uniforms_buffer(&render_data.uniforms_buffer, gpu_instance);
		render_pass.set_bind_group(1, *charmap_bind, &[]);
		render_pass.draw(0..3, 0..1);
	}
}
