use std::iter;
use std::str::FromStr;
use colorsys::{Hsl, Rgb};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
};
#[allow(unused_imports)]
use log::{Level, debug};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use crate::pipeline_context::{PipelineContext, Vertex};

mod pipeline_context;

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MyUniform {
    window_dims: [f32; 2],
    elapsed: f32,
    _pad: u32,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let log_level_str = option_env!("LOG_LEVEL").unwrap_or("debug");
            let log_level = Level::from_str(&log_level_str).unwrap();
            console_log::init_with_level(log_level).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    //BIG TODO: Custom error handling

    let event_loop = EventLoop::new().unwrap();
    let window = &WindowBuilder::new().build(&event_loop).unwrap();

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

    let pipeline_context = PipelineContext::new(window, wgpu::include_wgsl!("shader.wgsl"), size).await;

    let mut spice = 0;
    let mut bloop = 0;
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
            if let Some(bloop_q) = query_params.get("bloop") {
                if let Ok(bloop_i) = bloop_q.parse::<i32>() {
                    bloop = bloop_i;
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
        let color = rand_color(&mut rng);
        vertices.push(Vertex{
            position: [-side_len, side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color_saturated(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [-side_len, -side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color_saturated(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [side_len, -side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color_saturated(&mut rng) } else { color },
        });
        vertices.push(Vertex{
            position: [side_len, side_len - offset, size_step * i_f32],
            color: if should_gradient { rand_color_saturated(&mut rng) } else { color },
        });

        indices.push(0 + 4*i);
        indices.push(1 + 4*i);
        indices.push(3 + 4*i);
        indices.push(3 + 4*i);
        indices.push(1 + 4*i);
        indices.push(2 + 4*i);
    }

    let device = &pipeline_context.device;
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
    let output = pipeline_context.surface.get_current_texture().unwrap();

    let view = output //is this the viewscreen?
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform Buffer"),
        size: std::mem::size_of::<f32>() as u64 * 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: Some("uniform bind group"),
        layout: &pipeline_context.uniform_bg_layout,
        entries: &[wgpu::BindGroupEntry{
            binding: 0,
            resource: wgpu::BindingResource::Buffer(
                wgpu::BufferBinding{
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                },
            ),
        }],
    });

    pipeline_context.queue.write_buffer(
        &uniform_buffer,
        0,
        bytemuck::cast_slice(&[MyUniform{
            window_dims: [size.width as f32, size.height as f32],
            elapsed: 0.0,
            _pad: 0,
        }]),
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
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
        render_pass.set_pipeline(&pipeline_context.render_pipeline);
        render_pass.set_bind_group(0, &uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
    }

    // let pipeline_context = PipelineContext{
    //   device: ...,
    //   queue: ...,
    //   surface: ...,
    //   ...
    // }

    pipeline_context.queue.submit(iter::once(encoder.finish()));
    output.present();

    let start_time = get_time_millis();
    let _ = event_loop.run(move |event, window_target| fun_name(
        event,
        window_target,

        &pipeline_context,

        &vertex_buffer,
        &index_buffer,
        num_indices,

        &uniform_buffer,
        &uniform_bind_group,

        size,
        start_time,
    ));
}

fn fun_name(
    event: Event<()>,
    window_target: &winit::event_loop::EventLoopWindowTarget<()>,

    pipeline_context: &PipelineContext,

    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    num_indices: u32,

    uniform_buffer: &wgpu::Buffer,
    uniform_bind_group: &wgpu::BindGroup,

    size: winit::dpi::PhysicalSize<u32>,
    start_time: i64,
) {
    match event {
        Event::WindowEvent { event: WindowEvent::CloseRequested, window_id: _ } => {
            window_target.exit();
        }
        Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id: _ } => {
            let mut encoder = pipeline_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
            let elapsed = get_time_millis() - start_time;
            pipeline_context.queue.write_buffer(
                &uniform_buffer,
                0,
                bytemuck::cast_slice(&[MyUniform{
                    window_dims: [size.width as f32, size.height as f32],
                    elapsed: elapsed as f32,
                    _pad: 0,
                }]),
            );

            let output = pipeline_context.surface.get_current_texture().unwrap(); //could be a better name

            let view = output //is this the viewscreen?
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
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

                render_pass.set_pipeline(&pipeline_context.render_pipeline);
                render_pass.set_bind_group(0, &uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
                render_pass.draw_indexed(0..num_indices, 0, 0..1); // 2.
            }

            pipeline_context.queue.submit(iter::once(encoder.finish()));
            output.present();

            pipeline_context.window.request_redraw();
        }
        _ => {}
    };
}

fn rand_color(rng: &mut rand::rngs::ThreadRng) -> [f32; 3] {
    [rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()]
}

fn rand_color_saturated(rng: &mut rand::rngs::ThreadRng) -> [f32; 3] {
    let rgb: Rgb = Hsl::new(rng.gen::<f64>()*360.0, 100.0, 40.0, None).as_ref().into();
    let (r, g, b): (f32, f32, f32) = rgb.as_ref().into();
    [r/255.0, g/255.0, b/255.0]
}

fn get_time_millis() -> i64 {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use chrono::Utc;
            Utc::now().timestamp_millis()
        } else {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
        }
    }
}
