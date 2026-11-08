pub const CHAR_WIDTH: usize = 10;
pub const CHAR_HEIGHT: usize = 12;



pub mod load_textures;
pub use load_textures::*;
pub mod wgpu_defs;
pub use wgpu_defs::*;



pub(crate) use anyhow::*;



pub struct RenderData {
	
	pub char_atlas_tex: wgpu::Texture,
	pub char_atlas_view: wgpu::TextureView,
	pub color_lut_tex: wgpu::Texture,
	pub color_lut_view: wgpu::TextureView,
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



pub fn render(char_maps: &[(&wgpu::BindGroup, (u16, u16))], target_size: (u32, u32), render_pass: &mut wgpu::RenderPass, render_data: &mut RenderData, gpu_instance: &simple_gpu::GpuInstance) {
	render_pass.set_pipeline(&render_data.pipeline);
	render_pass.set_bind_group(0, &render_data.main_bind_group, &[]);
	for (charmap_bind, charmap_size) in char_maps {
		update_uniforms(&mut render_data.uniforms_buffer, target_size, *charmap_size);
		simple_gpu::sync_uniforms_buffer(&render_data.uniforms_buffer, &gpu_instance);
		render_pass.set_bind_group(1, *charmap_bind, &[]);
		render_pass.draw(0..3, 0..1);
	}
}
