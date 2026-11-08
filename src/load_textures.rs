use crate::{CHAR_HEIGHT, CHAR_WIDTH};
use anyhow::{Context, Result, bail};
use std::path::Path;



/// Loads up to 256 textures and places them correctly in an atlas. You probably shouldn't use this and just create the atlas manually
///
/// # Errors
///
/// This function returns an error if the `image` crate fails to load an image from path
pub fn load_all_textures(
	textures_path: impl AsRef<Path>,
	gpu_instance: &simple_gpu::GpuInstance,
) -> Result<wgpu::Texture> {
	let textures_path = textures_path.as_ref();
	#[allow(clippy::large_stack_arrays)]
	let mut atlas_data = [0; CHAR_WIDTH * CHAR_HEIGHT * 4 * 256];

	for i in 0..=255 {
		let path = textures_path.join(i.to_string() + ".png");
		if path.exists() {
			load_and_place_tex(&mut atlas_data, i, path)?;
		}
	}

	let texture = gpu_instance
		.wgpu_device
		.create_texture(&wgpu::TextureDescriptor {
			label: Some("Main Character Atlas"),
			size: wgpu::Extent3d {
				width: CHAR_WIDTH as u32 * 16,
				height: CHAR_HEIGHT as u32 * 16,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});
	gpu_instance.wgpu_queue.write_texture(
		wgpu::TexelCopyTextureInfo {
			texture: &texture,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		&atlas_data,
		wgpu::TexelCopyBufferLayout {
			offset: 0,
			bytes_per_row: Some(ATLAS_STRIDE as u32),
			rows_per_image: Some(CHAR_HEIGHT as u32 * 16),
		},
		wgpu::Extent3d {
			width: CHAR_WIDTH as u32 * 16,
			height: CHAR_HEIGHT as u32 * 16,
			depth_or_array_layers: 1,
		},
	);

	Ok(texture)
}



const CHAR_STRIDE: usize = CHAR_WIDTH * 4;
const ATLAS_STRIDE: usize = CHAR_STRIDE * 16;

/// Loads a texture from file and places it correctly in an atlas
///
/// # Errors
///
/// This function returns an error if the `image` crate fails to load an image from path
pub fn load_and_place_tex(
	atlas_data: &mut [u8; CHAR_WIDTH * CHAR_HEIGHT * 4 * 256],
	index: u8,
	path: impl AsRef<Path>,
) -> Result<()> {
	let path = path.as_ref();
	let texture_image =
		image::open(path).with_context(|| format!("Failed to read file {}", path.display()))?;
	let size = (texture_image.width(), texture_image.height());
	if size != (CHAR_WIDTH as u32, CHAR_HEIGHT as u32) {
		bail!(
			"Size of texture at {} must be 10x12, but is {}x{}",
			path.display(),
			size.0,
			size.1
		);
	}

	let tex_data = &*texture_image.into_rgba8();
	let atlas_row_pos =
		(index as usize & 0x0F) * CHAR_STRIDE + (index as usize >> 4) * ATLAS_STRIDE * CHAR_HEIGHT;
	unsafe {
		let mut tex_src = tex_data.as_ptr();
		let mut atlas_dest = atlas_data.as_mut_ptr().add(atlas_row_pos);
		for _row in 0..CHAR_HEIGHT {
			atlas_dest.copy_from(tex_src, CHAR_STRIDE);
			tex_src = tex_src.add(CHAR_STRIDE);
			atlas_dest = atlas_dest.add(ATLAS_STRIDE);
		}
	}

	Ok(())
}
