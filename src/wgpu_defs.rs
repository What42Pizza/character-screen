use crate::*;



pub fn make_main_bind_group_layout(gpu_instance: &simple_gpu::GpuInstance) -> wgpu::BindGroupLayout {
	gpu_instance.wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("character_screen Main Bind Group Layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				// texture
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: false },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				// sampler
				binding: 2,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
				count: None,
			},
		],
	})
}

pub fn make_char_map_bind_group_layout(gpu_instance: &simple_gpu::GpuInstance) -> wgpu::BindGroupLayout {
	gpu_instance.wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("character_screen Char Map Bind Group Layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry {
				// character map data storage
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage { read_only: true },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
	})
}



pub fn make_main_bind_group(main_bind_group_layout: &wgpu::BindGroupLayout, uniforms_buffer: &simple_gpu::UniformsBuffer<UniformsRawData>, char_atlas_view: &wgpu::TextureView, nearest_sampler: &wgpu::Sampler, gpu_instance: &simple_gpu::GpuInstance) -> wgpu::BindGroup {
	gpu_instance
		.wgpu_device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Main Bind Group"),
			layout: &main_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(uniforms_buffer.wgpu_buffer.as_entire_buffer_binding()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&char_atlas_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::Sampler(&nearest_sampler),
				},
			],
		})
}

pub fn make_char_map_bind_group(main_bind_group_layout: &wgpu::BindGroupLayout, character_map_buffer: &simple_gpu::GpuBuffer<u8>, gpu_instance: &simple_gpu::GpuInstance) -> wgpu::BindGroup {
	gpu_instance
		.wgpu_device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Char Map Bind Group"),
			layout: &main_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(character_map_buffer.wgpu_buffer.as_entire_buffer_binding()),
				},
			],
		})
}



pub fn make_pipeline_layout(main_bind_group_layout: &wgpu::BindGroupLayout, char_map_bind_group_layout: &wgpu::BindGroupLayout, gpu_instance: &simple_gpu::GpuInstance) -> wgpu::PipelineLayout {
	gpu_instance.wgpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Main Pipeline Layout"),
		bind_group_layouts: &[
			Some(&main_bind_group_layout),
			Some(&char_map_bind_group_layout),
		],
		immediate_size: 0,
	})
}

pub fn make_pipeline(main_shader: &simple_gpu::Shaders, pipeline_layout: &wgpu::PipelineLayout, target_format: wgpu::TextureFormat, gpu_instance: &simple_gpu::GpuInstance) -> wgpu::RenderPipeline {
	gpu_instance
		.wgpu_device
		.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Main Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: main_shader.vertex(),
				entry_point: None,
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: main_shader.fragment(),
				entry_point: None,
				targets: &[Some(wgpu::ColorTargetState {
					format: target_format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview_mask: None,
			cache: None,
		})
}



pub fn make_character_map_buffer(size: (u16, u16), gpu_instance: &simple_gpu::GpuInstance) -> simple_gpu::GpuBuffer<u8> {
	let buf_size  = size.0 as usize * size.1 as usize * 4;
	let mut character_map_buffer = simple_gpu::create_buffer::<u8>("Main Character Map Buffer", buf_size as u32, wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, gpu_instance);
	*character_map_buffer = vec![0; buf_size];
	for i in 0..128 {
		character_map_buffer[i*4] = i as u8;
	}
	//character_map_buffer[4] = 65;
	//character_map_buffer[5] = 1;
	//character_map_buffer[8] = 66;
	//character_map_buffer[9] = 1;
	simple_gpu::sync_buffer(&mut character_map_buffer, gpu_instance);
	character_map_buffer
}

pub fn make_character_map(size: (u16, u16), render_data: &RenderData, gpu_instance: &simple_gpu::GpuInstance) -> (simple_gpu::GpuBuffer<u8>, wgpu::BindGroup, (u16, u16)) {
	let buffer = make_character_map_buffer(size, gpu_instance);
	let bind_group = make_char_map_bind_group(&render_data.char_map_bind_group_layout, &buffer, &gpu_instance);
	(buffer, bind_group, size)
}



pub fn make_nearest_sampler(gpu_instance: &simple_gpu::GpuInstance) -> wgpu::Sampler {
	gpu_instance.wgpu_device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Nearest,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::MipmapFilterMode::Nearest,
		..Default::default()
	})
}



pub fn make_render_data(char_atlas_tex: wgpu::Texture, main_shader: simple_gpu::Shaders, output_format: wgpu::TextureFormat, gpu_instance: &simple_gpu::GpuInstance) -> RenderData {
	
	let char_atlas_view = char_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
	let uniforms_buffer = simple_gpu::create_uniforms_buffer::<UniformsRawData>(&gpu_instance);
	let nearest_sampler = make_nearest_sampler(&gpu_instance);
	let main_bind_group_layout = make_main_bind_group_layout(&gpu_instance);
	let char_map_bind_group_layout = make_char_map_bind_group_layout(&gpu_instance);
	let main_bind_group = make_main_bind_group(&main_bind_group_layout, &uniforms_buffer, &char_atlas_view, &nearest_sampler, &gpu_instance);
	let pipeline_layout = make_pipeline_layout(&main_bind_group_layout, &char_map_bind_group_layout, &gpu_instance);
	let pipeline = make_pipeline(&main_shader, &pipeline_layout, output_format, &gpu_instance);
	
	RenderData {
		char_atlas_tex,
		char_atlas_view,
		main_shader,
		uniforms_buffer,
		nearest_sampler,
		main_bind_group_layout,
		char_map_bind_group_layout,
		main_bind_group,
		pipeline_layout,
		pipeline,
		curr_pipeline_output_format: output_format,
	}
}
