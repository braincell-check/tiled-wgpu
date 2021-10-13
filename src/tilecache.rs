use std::{collections::HashMap, path::Path};

use tiled::Map;
use wgpu::{Device, Queue, Texture};

#[derive(Debug, Error)]
pub enum TileCacheError {
    #[error(transparent)]
    ImageLoad(#[from] image::ImageError),
}

#[derive(Debug)]
pub struct GpuTileCache {
    tileset_reference: HashMap<u32, Vec<Texture>>,
}

impl GpuTileCache {
    pub fn new<P: AsRef<Path>>(
        device: &mut Device,
        queue: &mut Queue,
        map: &Map,
        map_filepath: P,
    ) -> Result<Self, TileCacheError> {
        // Allocate storage for all textures
        let mut tileset_reference = HashMap::new();

        // Search for all tilesets used by the map
        for tileset in &map.tilesets {
            let textures = tileset
                .images
                .iter()
                .map(|image| {
                    // Get the image path relative to the map file
                    let image_path = map_filepath
                        .as_ref()
                        .parent()
                        .unwrap()
                        .join(image.source.clone());

                    // Load the image to memory
                    debug!("Loading tileset texture from: {}", image_path.display());
                    let image_data = image::open(image_path)?;
                    let diffuse_image_data = image_data.as_rgba8().unwrap();
                    let image_dimensions = diffuse_image_data.dimensions();

                    // Allocate the appropriate space in VRAM
                    let texture_extent = wgpu::Extent3d {
                        width: image_dimensions.0,
                        height: image_dimensions.1,
                        depth_or_array_layers: 1,
                    };
                    debug!(
                        "Allocating new tileset texture in VRAM with extent: {:?}",
                        texture_extent
                    );
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some(&image.source),
                        size: texture_extent,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    });

                    // Copy the loaded image from RAM to VRAM
                    debug!(
                        "Sending pixel buffer from RAM to VRAM for texture: {:?}",
                        image.source
                    );
                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        diffuse_image_data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: std::num::NonZeroU32::new(4 * image_dimensions.0),
                            rows_per_image: std::num::NonZeroU32::new(image_dimensions.1),
                        },
                        texture_extent,
                    );
                    Ok(texture)
                })
                .collect::<Vec<Result<Texture, TileCacheError>>>()
                .into_iter()
                .collect::<Result<Vec<Texture>, TileCacheError>>()?;

            // Store the texture in the tileset reference
            tileset_reference.insert(tileset.first_gid, textures);
        }

        Ok(Self { tileset_reference })
    }
}
