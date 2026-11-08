use crate::*;



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
