use crate::CharmapRenderData;
use std::ops::{Deref, DerefMut};



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

/// Misc data used for rendering charmaps
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CharmapRawUniforms {
	/// size of the current charmap
	pub charmap_size: [u32; 2],
}



impl Charmap {
	/// Creates a new charmap, same as [`create_charmap()`]
	#[cfg_attr(feature = "inline", inline)]
	pub fn new(
		name: impl Into<String>,
		size: (u16, u16),
		char_atlas_view: &wgpu::TextureView,
		colors_lut_view: &wgpu::TextureView,
		render_data: &CharmapRenderData,
		gpu_instance: &simple_gpu::GpuInstance,
	) -> Self {
		create_charmap(
			name,
			size,
			char_atlas_view,
			colors_lut_view,
			render_data,
			gpu_instance,
		)
	}

	/// Updates the character atlas texture and color LUT texture, same as [`update_charmap_textures()`]
	#[cfg_attr(feature = "inline", inline)]
	pub fn update_textures(
		charmap: &mut Self,
		char_atlas_view: &wgpu::TextureView,
		colors_lut_view: &wgpu::TextureView,
		render_data: &CharmapRenderData,
		gpu_instance: &simple_gpu::GpuInstance,
	) {
		update_charmap_textures(
			charmap,
			char_atlas_view,
			colors_lut_view,
			render_data,
			gpu_instance,
		);
	}

	/// Places a character (or generic value) at a position within the charmap
	#[cfg_attr(feature = "inline", inline)]
	pub fn place(&mut self, value: u8, pos: (u16, u16), colors: (u8, u8, u8)) {
		let i = (pos.0 as usize + pos.1 as usize * self.size.0 as usize) * 4;
		self.buffer[i] = value;
		self.buffer[i + 1] = colors.0;
		self.buffer[i + 2] = colors.1;
		self.buffer[i + 3] = colors.2;
	}
	/// Gets the character (or generic value) at a position within the charmap
	#[must_use]
	#[cfg_attr(feature = "inline", inline)]
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
	#[cfg_attr(feature = "inline", inline)]
	pub fn update_gpu(&mut self, gpu_instance: &simple_gpu::GpuInstance) {
		simple_gpu::sync_buffer(&mut self.buffer, gpu_instance);
	}
}



impl Deref for Charmap {
	type Target = Vec<u8>;
	#[cfg_attr(feature = "inline", inline)]
	fn deref(&self) -> &Self::Target {
		&self.buffer
	}
}

impl DerefMut for Charmap {
	#[cfg_attr(feature = "inline", inline)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.buffer
	}
}



/// Creates a new [`Charmap`], same as [`Charmap::new()`]
#[must_use]
#[cfg_attr(feature = "inline", inline)]
pub fn create_charmap(
	name: impl Into<String>,
	size: (u16, u16),
	char_atlas_view: &wgpu::TextureView,
	colors_lut_view: &wgpu::TextureView,
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
					resource: wgpu::BindingResource::TextureView(colors_lut_view),
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



/// Updates a [`Charmap`]'s character atlas texture and color LUT texture, same as [`Charmap::update_textures()`]
pub fn update_charmap_textures(
	charmap: &mut Charmap,
	char_atlas_view: &wgpu::TextureView,
	colors_lut_view: &wgpu::TextureView,
	render_data: &CharmapRenderData,
	gpu_instance: &simple_gpu::GpuInstance,
) {
	let bind_group = gpu_instance
		.wgpu_device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&charmap.name),
			layout: &render_data.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(
						charmap
							.uniforms_buffer
							.wgpu_buffer
							.as_entire_buffer_binding(),
					),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(char_atlas_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(colors_lut_view),
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&render_data.nearest_sampler),
				},
				wgpu::BindGroupEntry {
					binding: 4,
					resource: wgpu::BindingResource::Buffer(
						charmap.buffer.wgpu_buffer.as_entire_buffer_binding(),
					),
				},
			],
		});

	charmap.bind_group = bind_group;
}
