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
//! - Each character texture uses the red channel to mix the first two colors, then the green channel for mixing the first mix result with the third color, and that is the final fragment color
//! - Transparency is achieved using the colors LUT texture



/// This is the width of characters. It should be customizable but idk how to do that well
pub const CHAR_WIDTH: usize = 10;
/// This is the height of characters. It should be customizable but idk how to do that well
pub const CHAR_HEIGHT: usize = 14;



/// Functions for making and using charmaps
pub mod charmap;
pub use charmap::*;
/// Functions for initializing wgpu stuff
pub mod wgpu_defs;
pub use wgpu_defs::*;
/// Functions dealing with character texture atlases
pub mod atlas;
pub use atlas::*;



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



/// Does the actual rendering
#[cfg_attr(feature = "inline", inline)]
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
