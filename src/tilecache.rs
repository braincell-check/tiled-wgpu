use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tiled::Map;
use wgpu::{Device, Queue};

use crate::{
    bindgroup::build_tileset_bind_group_layout, error::TileCacheError, tiletex::TilesetTextureCache,
};

/// An on-GPU cache for tiled tilesets. One of these should exist per map.
#[derive(Debug)]
pub struct GpuTileCache {
    /// All tilesets
    tilesets: Vec<TilesetTextureCache>,
}

impl GpuTileCache {
    // Create a new tile cache from a tiled map.
    pub fn new<P: AsRef<Path> + Send + Sync>(
        device: &Device,
        queue: &Queue,
        map: &Map,
        map_filepath: P,
    ) -> Result<Self, TileCacheError> {
        // Search for all tilesets used by the map
        Ok(Self {
            tilesets: map
                .tilesets
                .par_iter()
                .map(|tileset| {
                    // For now, we only support one imate per tileset
                    if tileset.images.len() > 1 {
                        return Err(TileCacheError::TooManyImages);
                    }

                    // Process the first image
                    let image = tileset.images.first().unwrap();

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
                        label: Some(&format!("tileset:{}", image.source)),
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

                    // Build a bindgroup layout for the tileset
                    let bindgroup_layout = build_tileset_bind_group_layout(device);

                    Ok(TilesetTextureCache::new(
                        device,
                        tileset,
                        texture,
                        &bindgroup_layout,
                    ))
                })
                .collect::<Result<Vec<TilesetTextureCache>, TileCacheError>>()?,
        })
    }

    // / Get the tileset a specific tile belongs to
}
