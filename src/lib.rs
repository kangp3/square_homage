use std::iter;
use wgpu::{self, Adapter, SurfaceConfiguration};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
pub async fn run() {
    let temp_color = wgpu::Color {
        r: 0.0,
        g: 0.8,
        b: 0.0,
        a: 1.0,
    };

    env_logger::init();

    //BIG TODO: Custom error handling

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(&window).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps //this is so verbose, we can probably make it shorter, look into wgpu::Surface::get_default_config
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        desired_maximum_frame_latency: 2,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    surface.configure(&device, &config);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    let output = surface.get_current_texture().unwrap(); //could be a better name

    let view = output //is this the viewscreen?
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(temp_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    queue.submit(iter::once(encoder.finish()));
    output.present();

    event_loop.run(move |event, window| handle_event(event, window));
}

fn handle_event(event: Event<()>, window: &EventLoopWindowTarget<()>) {
    match event {
        Event::WindowEvent {
            event: ref window_event,
            window_id,
        } => match window_event {
            WindowEvent::CloseRequested => {
                window.exit();
            }
            _ => {}
        },
        _ => {}
    };
}
