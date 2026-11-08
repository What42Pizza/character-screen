use crate::*;
use std::path::Path;



pub fn load_all_textures(textures_path: impl AsRef<Path>, gpu_instance: &simple_gpu::GpuInstance) -> Result<wgpu::Texture> {
	let textures_path = textures_path.as_ref();
	let mut atlas_data = [0; CHAR_WIDTH * CHAR_HEIGHT * 4 * 256];
	
	load_and_place_tex(&mut atlas_data, 0, textures_path.join("double-line-cross.png"))?;
	load_and_place_tex(&mut atlas_data, 1, textures_path.join("double-line-horizontal.png"))?;
	load_and_place_tex(&mut atlas_data, 2, textures_path.join("double-line-vertical.png"))?;
	load_and_place_tex(&mut atlas_data, 32, textures_path.join("space.png"))?;
	load_and_place_tex(&mut atlas_data, 44, textures_path.join("comma.png"))?;
	load_and_place_tex(&mut atlas_data, 46, textures_path.join("period.png"))?;
	load_and_place_tex(&mut atlas_data, 47, textures_path.join("slash.png"))?;
	load_and_place_tex(&mut atlas_data, 58, textures_path.join("colon.png"))?;
	load_and_place_tex(&mut atlas_data, 59, textures_path.join("semi-colon.png"))?;
	load_and_place_tex(&mut atlas_data, 61, textures_path.join("equal.png"))?;
	load_and_place_tex(&mut atlas_data, 65, textures_path.join("A.png"))?;
	load_and_place_tex(&mut atlas_data, 66, textures_path.join("B.png"))?;
	load_and_place_tex(&mut atlas_data, 67, textures_path.join("C.png"))?;
	load_and_place_tex(&mut atlas_data, 68, textures_path.join("D.png"))?;
	load_and_place_tex(&mut atlas_data, 69, textures_path.join("E.png"))?;
	load_and_place_tex(&mut atlas_data, 70, textures_path.join("F.png"))?;
	load_and_place_tex(&mut atlas_data, 71, textures_path.join("G.png"))?;
	load_and_place_tex(&mut atlas_data, 72, textures_path.join("I.png"))?;
	load_and_place_tex(&mut atlas_data, 73, textures_path.join("J.png"))?;
	load_and_place_tex(&mut atlas_data, 74, textures_path.join("K.png"))?;
	load_and_place_tex(&mut atlas_data, 75, textures_path.join("L.png"))?;
	load_and_place_tex(&mut atlas_data, 76, textures_path.join("M.png"))?;
	load_and_place_tex(&mut atlas_data, 77, textures_path.join("N.png"))?;
	load_and_place_tex(&mut atlas_data, 78, textures_path.join("O.png"))?;
	load_and_place_tex(&mut atlas_data, 91, textures_path.join("left-square-bracket.png"))?;
	load_and_place_tex(&mut atlas_data, 92, textures_path.join("back-slash.png"))?;
	load_and_place_tex(&mut atlas_data, 93, textures_path.join("right-square-bracket.png"))?;
	load_and_place_tex(&mut atlas_data, 95, textures_path.join("underscore.png"))?;
	
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

pub fn load_and_place_tex(atlas_data: &mut [u8; CHAR_WIDTH * CHAR_HEIGHT * 4 * 256], index: u8, path: impl AsRef<Path>) -> Result<()> {
	
	let path = path.as_ref();
	let texture_image =
		image::open(path).with_context(|| format!("Failed to read file {}", path.display()))?;
	let size = (texture_image.width(), texture_image.height());
	if size != (CHAR_WIDTH as u32, CHAR_HEIGHT as u32) {
		bail!("Size of texture at {path:?} must be 10x12, but is {}x{}", size.0, size.1);
	}
	
	let tex_data = &*texture_image.into_rgba8();
	let atlas_row_pos = (index as usize & 0x0F) * CHAR_STRIDE + (index as usize >> 4) * ATLAS_STRIDE * CHAR_HEIGHT;
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
