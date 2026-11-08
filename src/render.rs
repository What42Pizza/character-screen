use crate::*;



pub fn prepare_rendering(target_size: (u32, u32), render_pass: &mut wgpu::RenderPass, render_data: &mut RenderData, gpu_instance: &simple_gpu::GpuInstance) {
	
	update_uniforms(&mut render_data.uniforms_buffer, target_size);
	simple_gpu::sync_uniforms_buffer(&render_data.uniforms_buffer, &gpu_instance);
	
	render_pass.set_pipeline(&render_data.pipeline);
	render_pass.set_bind_group(0, &render_data.main_bind_group, &[]);
	
}

pub fn render(char_map_bind_group: &wgpu::BindGroup, render_pass: &mut wgpu::RenderPass, gpu_instance: &simple_gpu::GpuInstance) {
	render_pass.set_bind_group(1, char_map_bind_group, &[]);
	render_pass.draw(0..3, 0..1);
}
