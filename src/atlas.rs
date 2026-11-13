#[cfg(feature = "image")]
use anyhow::{Context, anyhow};
use anyhow::{Result, bail};
#[cfg(feature = "image")]
use std::{ffi::OsStr, path::Path};

use crate::{CHAR_HEIGHT, CHAR_WIDTH};



/// Loads a character texture atlas from a file path
///
/// The texture is expected to be 175x239 (10x14 characters textures with a one pixel gap in between), and the returned texture is 160x224 (10x16 chars with no gaps)
///
/// # Errors
///
/// This errors if `image::open` fails or if the size of the image is not exactly 175x239
#[cfg(feature = "image")]
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

	let texture_data_rgba = texture_image.into_rgba8();
	let mut texture_data = Vec::with_capacity(texture_data_rgba.len() / 2);
	for i in 0..texture_data_rgba.len() / 4 {
		texture_data.push(texture_data_rgba.as_raw()[i * 4]);
		texture_data.push(texture_data_rgba.as_raw()[i * 4 + 1]);
	}
	let texture_data = remove_padding(&mut texture_data)?;

	let texture = simple_gpu::create_texture(
		file_name,
		(160, 224),
		wgpu::TextureFormat::Rg8Unorm,
		wgpu::FilterMode::Nearest,
		1,
		gpu_instance,
	);
	simple_gpu::update_texture(&texture, texture_data, gpu_instance);

	Ok(texture)
}



/// Removes the padding from a character atlas texture
///
/// The input texture must be RG8, 175x239 (`CHAR_SIZE * 16 + 15`, one pixel gap between each character texture), and the data is packed into an RG8 160x224 texture. This both mutates the given data and returns a slice for the compacted data
///
/// # Errors
///
/// This errors if `data.len()` is not exactly 167300 (which is 175*239*4)
pub fn remove_padding(data: &mut [u8]) -> Result<&[u8]> {
	const CHAR_STRIDE: usize = CHAR_WIDTH * 2;
	const EXPECTED_DATA_LEN: usize = (CHAR_WIDTH * 16 + 15) * (CHAR_HEIGHT * 16 + 15) * 2;
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
					src = src.add(CHAR_STRIDE + 2);
					dst = dst.add(CHAR_STRIDE);
				}
				src = src.sub(2);
			}
			src = src.add(CHAR_STRIDE * 16 + 15 * 2);
		}
	}
	Ok(&data[..160 * 224 * 2])
}
