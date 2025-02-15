mod bar;
mod style;
mod text;

use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use text::{ab_glyph, GlyphBrushBuilder, Section, Text};
use tty::{pty, COLS, ROWS};
use wgpu::util::DeviceExt;
use winit::{event, event_loop};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = event_loop::EventLoop::new();

    let window_builder = style::create_window_builder("Rio");
    let window = window_builder.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    env::set_var("TERM", "rio");

    let (device, queue) = (async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Request device")
    })
    .await;

    let mut staging_belt = wgpu::util::StagingBelt::new(64);
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("bar/bar.wgsl").into()),
    });

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
        },
    );

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(bar::VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(bar::INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = bar::INDICES.len() as u32;

    let font = ab_glyph::FontArc::try_from_slice(style::FONT_FIRA_MONO)?;
    let mut brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    // let command_intro: String = String::from("■ ~ "); // ▲

    let shell = Cow::Borrowed("bash");
    let (process, mut w_process, _pid) = pty(&shell, COLS as u16, ROWS as u16);

    // println!("{:?}", pid);
    // ■ ~
    let output: Arc<Mutex<String>> = Arc::new(Mutex::from(String::from("")));
    let message = Arc::clone(&output);
    tokio::spawn(async move {
        let reader = BufReader::new(process);

        // for ou in reader.lines() {
        //     println!("{:?}", ou.as_ref().unwrap());
        //     let mut a = message.lock().unwrap();
        //     *a = String::from(format!("{} {} \n", *a, ou.unwrap()));
        // }

        for input_byte in reader.bytes() {
            let ib = [input_byte.unwrap()];
            let ns = String::from_utf8_lossy(&ib);
            let mut a = message.lock().unwrap();
            *a = String::from(format!("{}{}", *a, ns));
        }
    });

    w_process.write_all(b"").unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            event::Event::WindowEvent {
                event: event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = event_loop::ControlFlow::Exit,

            event::Event::WindowEvent {
                event:
                    event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // println!("{:?}", keycode);
                        match keycode {
                            event::VirtualKeyCode::E => {
                                w_process.write_all(b"e").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::C => {
                                w_process.write_all(b"c").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::H => {
                                w_process.write_all(b"h").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::O => {
                                w_process.write_all(b"o").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::R => {
                                w_process.write_all(b"r").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::I => {
                                w_process.write_all(b"i").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::L => {
                                w_process.write_all(b"l").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::S => {
                                w_process.write_all(b"s").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::W => {
                                w_process.write_all(b"w").unwrap();
                                // window.request_redraw();
                            }
                            event::VirtualKeyCode::Space => {
                                w_process.write_all(b" ").unwrap();
                                // window.request_redraw();
                            }
                            // event::VirtualKeyCode::Space => {
                            //     command_text.push_str(" ");
                            // window.request_redraw();
                            // }
                            event::VirtualKeyCode::Return => {
                                w_process.write_all(b"\n").unwrap();
                                // window.request_redraw();
                            }
                            _ => {
                                println!("code not implemented");
                            }
                        }

                        window.request_redraw();
                    }
                    winit::event::ElementState::Released => {
                        window.request_redraw();
                    }
                }
                // Render only text as typing
            }

            event::Event::WindowEvent {
                event: event::WindowEvent::Focused(focused),
                ..
            } => {
                if focused {
                    // TODO: Optmize non-focused rendering perf
                }
            }

            event::Event::WindowEvent {
                event: event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::AutoVsync,
                    },
                );

                window.request_redraw();
            }
            event::Event::RedrawRequested { .. } => {
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Redraw"),
                    });

                let frame = surface.get_current_texture().expect("Get next frame");
                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let render_pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

                let render_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[bar::Vertex::desc()],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: render_format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                            polygon_mode: wgpu::PolygonMode::Fill,
                            // Requires Features::DEPTH_CLIP_CONTROL
                            unclipped_depth: false,
                            // Requires Features::CONSERVATIVE_RASTERIZATION
                            conservative: false,
                        },
                        depth_stencil: None, // 1.
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                    });

                {
                    let mut render_pass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Clear frame"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        style::DEFAULT_COLOR_BACKGROUND,
                                    ),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });

                    render_pass.set_pipeline(&render_pipeline); // 2.
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.draw(0..num_indices, 0..1);
                }

                {
                    brush.queue(Section {
                        screen_position: (24.0, 120.0),
                        bounds: ((size.width - 40) as f32, size.height as f32),
                        text: vec![Text::new(&output.lock().unwrap())
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(36.0)],
                        ..Section::default()
                    });

                    brush
                        .draw_queued(
                            &device,
                            &mut staging_belt,
                            &mut encoder,
                            view,
                            size.width,
                            size.height,
                        )
                        .expect("Draw queued");
                }

                staging_belt.finish();
                queue.submit(Some(encoder.finish()));
                frame.present();

                // Recall unused staging buffers
                staging_belt.recall();
            }
            _ => {
                *control_flow = event_loop::ControlFlow::Wait;
            }
        }
    })
}
