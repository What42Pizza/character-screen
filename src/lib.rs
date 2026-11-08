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



use anyhow::{Context, Result, anyhow, bail};
use std::{ffi::OsStr, path::Path};



/// Everything this crate needs for rendering character maps
pub struct RenderData {
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



/// Holds all the data for a character map, with all character data being stored in `Self.buffer`
pub struct Charmap {
	/// Holds the name, this is given to wgpu for easier debugging
	pub name: String,
	/// The size that this charmap was created as
	pub size: (u16, u16),
	/// Holds the size of the charmap
	pub uniforms_buffer: simple_gpu::UniformsBuffer<UniformsRawData>,
	/// Holds both the cpu copy and (a handle for) the gpu copy of the charmap data
	pub buffer: simple_gpu::GpuBuffer<u8>,
	/// Holds the resources that are given to the shader
	pub bind_group: wgpu::BindGroup,
}



/// Does the actual rendering
#[inline]
pub fn render(
	charmaps: &[&Charmap],
	render_pass: &mut wgpu::RenderPass,
	render_data: &mut RenderData,
) {
	render_pass.set_pipeline(&render_data.pipeline);
	for charmap in charmaps {
		render_pass.set_bind_group(0, &charmap.bind_group, &[]);
		render_pass.draw(0..3, 0..1);
	}
}



/// Loads a character texture atlas from a file path
///
/// The texture is expected to be 175x239 (10x14 characters textures with a one pixel gap in between), and the returned texture is 160x224 (10x16 chars with no gaps)
///
/// # Errors
///
/// This errors if `image::open` fails or if the size of the image is not exactly 175x239
pub fn load_character_atlas(
	path: impl AsRef<Path>,
	gpu_instance: &simple_gpu::GpuInstance,
) -> Result<simple_gpu::Texture> {
	let path = path.as_ref();

	let texture_image =
		image::open(path).with_context(|| format!("Failed to read file {}", path.display()))?;
	let size = (texture_image.width(), texture_image.height());
	if size != (175, 239) {
		bail!(
			"Character atlas must be 175x239 (10x14 character textures with a one pixel gap between each character), but the actual size is {}x{}",
			size.0,
			size.1
		);
	}
	let file_name = path
		.file_name()
		.map(OsStr::to_string_lossy)
		.ok_or_else(|| anyhow!("Failed to get file name from path {}", path.display()))?;

	let mut texture_data = texture_image.into_rgba8();
	let texture_data = remove_padding(&mut texture_data)?;

	let texture = simple_gpu::create_texture(
		file_name,
		(160, 224),
		wgpu::TextureFormat::Rgba8Unorm,
		wgpu::FilterMode::Nearest,
		1,
		gpu_instance,
	);
	simple_gpu::update_texture(&texture, texture_data, gpu_instance);

	Ok(texture)
}



/// Removes the padding from a character atlas texture
///
/// The input texture must be RGBA8, 175x239 (`CHAR_SIZE * 16 + 15`, one pixel gap between each character texture), and the data is packed into an RGBA8 160x224 texture. This both mutates the given data and returns a slice for the compacted data
///
/// # Errors
///
/// This errors if `data.len()` is not exactly 167300 (which is 175*239*4)
pub fn remove_padding(data: &mut [u8]) -> Result<&[u8]> {
	const CHAR_STRIDE: usize = CHAR_WIDTH * 4;
	const EXPECTED_DATA_LEN: usize = (CHAR_WIDTH * 16 + 15) * (CHAR_HEIGHT * 16 + 15) * 4;
	if data.len() != EXPECTED_DATA_LEN {
		bail!(
			"Expected input character texture atlas to be {} bytes (175x239, 10x14 characters with a one pixel gap between each character), but the actual length is {}",
			EXPECTED_DATA_LEN,
			data.len()
		);
	}

	unsafe {
		let mut src = data.as_ptr();
		let mut dst = data.as_mut_ptr();
		for _char_y in 0..16 {
			for _in_char_y in 0..CHAR_HEIGHT {
				for _char_x in 0..16 {
					dst.copy_from(src, CHAR_STRIDE);
					src = src.add(CHAR_STRIDE + 4);
					dst = dst.add(CHAR_STRIDE);
				}
				src = src.sub(4);
			}
			src = src.add(CHAR_STRIDE * 16 + 15 * 4);
		}
	}
	Ok(&data[..160 * 224 * 4])
}
