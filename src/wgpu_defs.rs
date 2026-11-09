#![allow(missing_docs)]

use crate::{CHAR_HEIGHT, CHAR_WIDTH, Charmap, CharmapRawUniforms, CharmapRenderData};



#[must_use]
#[inline]
pub fn make_bind_group_layout(gpu_instance: &simple_gpu::GpuInstance) -> wgpu::BindGroupLayout {
	gpu_instance
		.wgpu_device
		.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("character_screen Main Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					// uniforms
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
					// char atlas texture
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
					// color LUT texture
					binding: 2,
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
					binding: 3,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					// character map data storage
					binding: 4,
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



#[must_use]
#[inline]
pub fn make_pipeline_layout(
	bind_group_layout: &wgpu::BindGroupLayout,
	gpu_instance: &simple_gpu::GpuInstance,
) -> wgpu::PipelineLayout {
	gpu_instance
		.wgpu_device
		.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Main Pipeline Layout"),
			bind_group_layouts: &[Some(bind_group_layout)],
			immediate_size: 0,
		})
}

#[must_use]
#[inline]
pub fn make_pipeline(
	main_shader: &simple_gpu::Shaders,
	pipeline_layout: &wgpu::PipelineLayout,
	target_format: wgpu::TextureFormat,
	gpu_instance: &simple_gpu::GpuInstance,
) -> wgpu::RenderPipeline {
	gpu_instance
		.wgpu_device
		.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Main Pipeline"),
			layout: Some(pipeline_layout),
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



#[must_use]
#[inline]
pub fn make_character_map_buffer(
	size: (u16, u16),
	gpu_instance: &simple_gpu::GpuInstance,
) -> simple_gpu::GpuBuffer<u8> {
	let buf_size = size.0 as usize * size.1 as usize * 4;
	let mut character_map_buffer = simple_gpu::create_buffer::<u8>(
		"Main Character Map Buffer",
		buf_size as u32,
		wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
		gpu_instance,
	);
	*character_map_buffer = vec![32; buf_size];
	simple_gpu::sync_buffer(&mut character_map_buffer, gpu_instance);
	character_map_buffer
}



#[must_use]
#[inline]
pub fn create_charmap(
	name: impl Into<String>,
	size: (u16, u16),
	char_atlas_view: &wgpu::TextureView,
	color_lut_view: &wgpu::TextureView,
	render_data: &CharmapRenderData,
	gpu_instance: &simple_gpu::GpuInstance,
) -> Charmap {
	let name = name.into();

	let mut uniforms_buffer = simple_gpu::create_uniforms_buffer(gpu_instance);
	uniforms_buffer.cpu_copy = CharmapRawUniforms {
		charmap_size: [u32::from(size.0), u32::from(size.1)],
	};
	simple_gpu::sync_uniforms_buffer(&uniforms_buffer, gpu_instance);

	let mut buffer = simple_gpu::create_buffer(
		&name,
		u32::from(size.0) * u32::from(size.1) * 4,
		wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
		gpu_instance,
	);
	*buffer = vec![0; size.0 as usize * size.1 as usize * 4];
	simple_gpu::sync_buffer(&mut buffer, gpu_instance);

	let bind_group = gpu_instance
		.wgpu_device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&name),
			layout: &render_data.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(
						uniforms_buffer.wgpu_buffer.as_entire_buffer_binding(),
					),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(char_atlas_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(color_lut_view),
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&render_data.nearest_sampler),
				},
				wgpu::BindGroupEntry {
					binding: 4,
					resource: wgpu::BindingResource::Buffer(
						buffer.wgpu_buffer.as_entire_buffer_binding(),
					),
				},
			],
		});

	Charmap {
		name,
		size,
		uniforms_buffer,
		buffer,
		bind_group,
	}
}



#[must_use]
#[inline]
pub fn make_nearest_sampler(gpu_instance: &simple_gpu::GpuInstance) -> wgpu::Sampler {
	gpu_instance
		.wgpu_device
		.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::MipmapFilterMode::Nearest,
			..Default::default()
		})
}



#[must_use]
#[inline]
pub fn make_render_data(
	output_format: wgpu::TextureFormat,
	gpu_instance: &simple_gpu::GpuInstance,
) -> CharmapRenderData {
	let main_shader = simple_gpu::make_glsl_shaders(
		"shaders/main.vsh",
		include_str!("shaders/main.vsh"),
		"shaders/main.fsh",
		include_str!("shaders/main.fsh"),
		gpu_instance,
		&[
			("CHAR_WIDTH", &CHAR_WIDTH.to_string()),
			("CHAR_HEIGHT", &CHAR_HEIGHT.to_string()),
		],
	);

	let nearest_sampler = make_nearest_sampler(gpu_instance);
	let bind_group_layout = make_bind_group_layout(gpu_instance);
	let pipeline_layout = make_pipeline_layout(&bind_group_layout, gpu_instance);
	let pipeline = make_pipeline(&main_shader, &pipeline_layout, output_format, gpu_instance);

	CharmapRenderData {
		main_shader,
		nearest_sampler,
		bind_group_layout,
		pipeline_layout,
		pipeline,
		curr_pipeline_output_format: output_format,
	}
}
