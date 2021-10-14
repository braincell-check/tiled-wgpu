use std::path::Path;

use log::{error, info};
use pixels::{Pixels, SurfaceTexture};
use tiled_wgpu::tilecache::GpuTileCache;
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[tokio::main]
pub async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .init();

    // Build the window loop and manager
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(1000.0, 600.0);
        WindowBuilder::new()
            .with_title("Tiled Map Load")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // Set up Pixels FB
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(1000, 600, surface_texture).unwrap()
    };

    // Create the logical device and command queue
    let mut device = pixels.device();
    let mut queue = pixels.queue();

    // Load an example map using tiled
    let map_path = Path::new("./tiled/examples/desert.tmx");
    let map = tiled::parse_file(map_path).unwrap();

    // Load the map into VRAM
    let map_tilecache = GpuTileCache::new(&mut device, &mut queue, &map, map_path).unwrap();

    // Dump the map
    info!("Map sent to GPU: {:#?}", map_tilecache);

    // Render loop
    event_loop.run(move |event, _, control_flow| {
        // Handle control events
        match event {
            Event::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => {}
        };
    });
}
