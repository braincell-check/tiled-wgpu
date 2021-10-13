use std::path::Path;

use log::info;
use tiled_wgpu::tilecache::GpuTileCache;

#[tokio::main]
pub async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .init();

    // Init a wgpu instance
    let adapter = wgpu::Instance::new(wgpu::Backends::all())
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    // Dump some info about the GPU adapter
    info!("GPU Adapter: {:?}", adapter.get_info());

    // Create the logical device and command queue
    let (mut device, mut queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load an example map using tiled
    let map_path = Path::new("./tiled/examples/desert.tmx");
    let map = tiled::parse_file(map_path).unwrap();

    // Load the map into VRAM
    let map_tilecache = GpuTileCache::new(&mut device, &mut queue, &map, map_path).unwrap();

    // Dump the map
    info!("Map sent to GPU: {:#?}", map_tilecache);
}
