use std::iter;
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};
#[allow(unused_imports)]
use log::debug;
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    //BIG TODO: Custom error handling

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::WindowExtWebSys;
            let web_window = web_sys::window().unwrap();
            web_window.document()
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("body")?;
                    let canvas = window.canvas().unwrap();
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
            .expect("Couldn't append canvas to document body.");

            // TODO(peter): For some reason inner_size in web is returning 0, 0 (maybe the size of
            // the canvas? So instead we get the size of the entire window
            use winit::dpi::PhysicalSize;
            let size = {
                let width = web_window.inner_width().unwrap().as_f64().unwrap() as u32;
                let height = web_window.inner_height().unwrap().as_f64().unwrap() as u32;
                PhysicalSize{
                    width,
                    height,
                }
            };
        } else {
            let size = window.inner_size();
        }
    }

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
                required_limits: if cfg!(target_arch = "wasm32") { wgpu::Limits::downlevel_webgl2_defaults() } else { wgpu::Limits::default() },
            },
            None,
        )
        .await
        .unwrap();

    surface.configure(&device, &config);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    //VERTEX things
    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
    }

    impl Vertex {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        fn desc() -> wgpu::VertexBufferLayout<'static> {
            //is this a good name?
            use std::mem;

            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &Self::ATTRIBS,
            }
        }
    }

    let mut spice = 0;
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let loc = web_sys::window().unwrap().location();
            let search = loc.search().unwrap();
            let query_params = web_sys::UrlSearchParams::new_with_str(&search)
                .unwrap();
            if let Some(spice_q) = query_params.get("spice") {
                if let Ok(spice_i) = spice_q.parse::<i32>() {
                    spice = spice_i;
                }
            }
        }
    }

    let mut rng = rand::thread_rng();

    let gradient_thresh = 1.0 - 0.25 * spice as f32;

    let size_step = 0.1835;
    let pos_step = 0.0918;
    let mut vertices = vec![];
    let mut indices: Vec<u16> = vec![];
    for i in 0..4 {
        let should_gradient = gradient_thresh <= 0.0 || rng.gen::<f32>() > gradient_thresh;

        let i_f32 = i as f32;
        let side_len = 0.9 - i_f32 * size_step;
        let offset = pos_step * i_f32;
        // let height = if is_wider { side_len } else { side_len * aspect_ratio_recip };
        // let width = if is_wider { side_len * aspect_ratio_recip } else { side_len };
        // let offset = if is_wider { pos_step * i_f32 } else { pos_step * i_f32 * aspect_ratio_recip };
        let color = rand_color(&mut rng);
        vertices.push(Vertex{
            position: [-side_len, side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [-side_len, -side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [side_len, -side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [side_len, side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color(&mut rng) } else { color },
        });

        indices.push(0 + 4*i);
        indices.push(1 + 4*i);
        indices.push(3 + 4*i);
        indices.push(3 + 4*i);
        indices.push(1 + 4*i);
        indices.push(2 + 4*i);
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = indices.len() as u32;
    let output = surface.get_current_texture().unwrap(); //could be a better name

    let view = output //is this the viewscreen?
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let ar_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Aspect ratio uniform Buffer"),
        size: std::mem::size_of::<f32>() as u64 * 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ar_uniform_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: Some("AR Uniform BG layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry{
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let ar_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: Some("Aspect ratio uniform bind group"),
        layout: &ar_uniform_bg_layout,
        entries: &[wgpu::BindGroupEntry{
            binding: 0,
            resource: wgpu::BindingResource::Buffer(
                wgpu::BufferBinding{
                    buffer: &ar_uniform_buffer,
                    offset: 0,
                    size: None,
                },
            ),
        }],
    });
    queue.write_buffer(&ar_uniform_buffer, 0, bytemuck::cast_slice(&[size.width as f32, size.height as f32, size.width as f32, size.height as f32]));

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl")); //TODO: Break this string out to be called by main

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &ar_uniform_bg_layout,
        ],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //copy pasted from learnWGPU. This is verbose
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // TODO(peter): Figure out how to actually get the canvas to load with an alpha
                    // channel, or potentially add another set of vertices just to draw a
                    // transparent layer on the background.
                    load: wgpu::LoadOp::Clear(wgpu::Color{
                        r: 0.8,
                        g: 0.8,
                        b: 0.8,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(0, &ar_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
    }

    queue.submit(iter::once(encoder.finish()));
    output.present();

    let _ = event_loop.run(move |event, window| handle_event(event, window));
}

fn rand_color(rng: &mut rand::rngs::ThreadRng) -> [f32; 3] {
    [rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()]
}

fn handle_event(event: Event<()>, window: &EventLoopWindowTarget<()>) {
    match event {
        Event::WindowEvent {
            event: ref window_event,
            window_id: _,
        } => match window_event {
            WindowEvent::CloseRequested => {
                window.exit();
            }
            _ => {}
        },
        _ => {}
    };
}
