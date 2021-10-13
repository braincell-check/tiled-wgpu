use std::{collections::HashMap, ops::Add};

use nalgebra::Vector2;
use parry2d::bounding_volume::AABB;
use tiled::Tileset;
use wgpu::{Device, Sampler, SamplerDescriptor, Texture, TextureView, TextureViewDimension};

/// Contains all the data needed to tell the GPU about a single tiled tileset
#[derive(Debug)]
pub struct TilesetTextureCache {
    /// The base GID of this tileset
    pub base_gid: u32,
    /// The texture containing the whole tileset
    pub texture: Texture,
    /// The sampler to use when sampling the texture
    pub sampler: Sampler,
    /// The view for the texture
    pub view: TextureView,
    /// A map between tile ids and their positions in the texture
    pub tile_bounds: HashMap<u32, AABB>,
}

impl TilesetTextureCache {
    /// Construct a new tileset texture from a tileset.
    pub fn new(device: &mut Device, tileset: &Tileset, texture: Texture) -> Self {
        // Calculate the number of tiles that span the width of the tileset
        let tiles_per_row =
            tileset.images.first().unwrap().width as u32 / (tileset.tile_width + tileset.spacing);

        Self {
            base_gid: tileset.first_gid,
            sampler: device.create_sampler(&SamplerDescriptor {
                label: Some("tile_sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            view: texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("tileset:{}:view", tileset.first_gid)),
                dimension: Some(TextureViewDimension::D2),
                ..Default::default()
            }),
            texture,
            tile_bounds: tileset
                .tiles
                .iter()
                .enumerate()
                .map(|(index, tile)| {
                    (tile.id + tileset.first_gid, {
                        let top_left = Vector2::new(
                            tileset.margin as f32
                                + (index as u32 % tiles_per_row) as f32
                                    * (tileset.tile_width + tileset.spacing) as f32,
                            tileset.margin as f32
                                + (index as u32 / tiles_per_row) as f32
                                    * (tileset.tile_height + tileset.spacing) as f32,
                        );

                        AABB::new(
                            top_left.into(),
                            top_left
                                .add(Vector2::new(
                                    tileset.tile_width as f32,
                                    tileset.tile_height as f32,
                                ))
                                .into(),
                        )
                    })
                })
                .collect(),
        }
    }
}
