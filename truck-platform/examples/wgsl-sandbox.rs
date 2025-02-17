//! A sample of creating a render object by implementing "Rendered" in a new structure.
//!
//! One can use xyr WGSL shader in the following way:
//!
//! - Enter the shader path as an argument when executing the program.
//! - Drag and drop the shader into the window.
//!
//! The rule of shaders:
//!
//! - One can draw a image by implementing the function:
//!
//! ```wgsl
//! vec4<f32> main_image(coord: vec2<f32>, env: Environment);
//! ```
//!
//! - The parameter `coord` is the fragment coordinate. The origin is the lower left.
//! - The parameter `env` has the environment information. The declaration of struct is the following:
//!
//! ```wgsl
//! struct Environment {
//!     resolution: vec2<f32>;  // the resolution of the image
//!     mouse: vec4<f32>;       // the mouse information behaving the same as `iMouse` in Shadertoy.
//!     time: f32;              // the number of seconds since the application started.
//! };
//! ```
//!
//! Also, see the sample `newton-cuberoot.wgsl`, default shader, in `examples`.

use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

/// minimum example for implementing `Rendered`.
mod plane {
    use super::*;

    /// Canvas to draw by fragment shader.
    pub struct Plane {
        module: ShaderModule,
        pub mouse: [f32; 4],
        id: RenderID,
    }

    const BASE_PREFIX: &str = "[[block]]
struct SceneInfo {
    time: f32;
    nlights: u32;
};

[[block]]
struct Resolution {
    resolution: vec2<f32>;
};

[[block]]
struct Mouse {
    mouse: vec4<f32>;
};

[[group(0), binding(2)]]
var<uniform> __info: SceneInfo;

[[group(1), binding(0)]]
var<uniform> __resolution: Resolution;

[[group(1), binding(1)]]
var<uniform> __mouse: Mouse;

struct Environment {
    resolution: vec2<f32>;
    mouse: vec4<f32>;
    time: f32;
};

";

    const BASE_SHADER: &str = "[[stage(vertex)]]
fn vs_main([[location(0)]] idx: u32) -> [[builtin(position)]] vec4<f32> {
    var vertex: array<vec2<f32>, 4>;
    vertex[0] = vec2<f32>(-1.0, -1.0);
    vertex[1] = vec2<f32>(1.0, -1.0);
    vertex[2] = vec2<f32>(-1.0, 1.0);
    vertex[3] = vec2<f32>(1.0, 1.0);
    return vec4<f32>(vertex[idx], 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var env: Environment;
    env.resolution = __resolution.resolution;
    env.mouse = __mouse.mouse;
    env.time = __info.time;
    let coord = vec2<f32>(
        position.x,
        __resolution.resolution.y - position.y,
    );
    return main_image(coord, env);
}
";

    // Implementation of Rendered for Plane.
    impl Rendered for Plane {
        // `Rendered::render_id()` can be implemented by macro.
        impl_render_id!(id);

        // Vertices: [0, 1, 2, 2, 1, 3] as [u32; 6].
        // There is not the index buffer.
        fn vertex_buffer(
            &self,
            handler: &DeviceHandler,
        ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
            let vertex_buffer = BufferHandler::from_slice(
                &[0 as u32, 1, 2, 2, 1, 3],
                handler.device(),
                BufferUsages::VERTEX,
            );
            (Arc::new(vertex_buffer), None)
        }

        // bind group is only one uniform buffer: resolution
        fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
            Arc::new(
                handler
                    .device()
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStages::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStages::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    }),
            )
        }
        // bind group is only one uniform buffer: resolution
        fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
            let config = handler.config();
            let resolution = [config.width as f32, config.height as f32];
            Arc::new(bind_group_util::create_bind_group(
                handler.device(),
                layout,
                vec![
                    BufferHandler::from_slice(&resolution, handler.device(), BufferUsages::UNIFORM)
                        .binding_resource(),
                    BufferHandler::from_slice(&self.mouse, handler.device(), BufferUsages::UNIFORM)
                        .binding_resource(),
                ],
            ))
        }

        // Describe pipeline
        fn pipeline(
            &self,
            handler: &DeviceHandler,
            layout: &PipelineLayout,
            sample_count: u32,
        ) -> Arc<RenderPipeline> {
            let config = handler.config();
            Arc::new(
                handler
                    .device()
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        layout: Some(layout),
                        vertex: VertexState {
                            module: &self.module,
                            entry_point: "vs_main",
                            buffers: &[VertexBufferLayout {
                                array_stride: std::mem::size_of::<u32>() as BufferAddress,
                                step_mode: VertexStepMode::Vertex,
                                attributes: &[VertexAttribute {
                                    format: VertexFormat::Uint32,
                                    offset: 0,
                                    shader_location: 0,
                                }],
                            }],
                        },
                        fragment: Some(FragmentState {
                            module: &self.module,
                            entry_point: "fs_main",
                            targets: &[ColorTargetState {
                                format: config.format,
                                blend: Some(BlendState::REPLACE),
                                write_mask: ColorWrites::ALL,
                            }],
                        }),
                        primitive: PrimitiveState {
                            topology: PrimitiveTopology::TriangleList,
                            front_face: FrontFace::Ccw,
                            cull_mode: Some(Face::Back),
                            polygon_mode: PolygonMode::Fill,
                            clamp_depth: false,
                            ..Default::default()
                        },
                        depth_stencil: Some(DepthStencilState {
                            format: TextureFormat::Depth32Float,
                            depth_write_enabled: true,
                            depth_compare: wgpu::CompareFunction::Less,
                            stencil: Default::default(),
                            bias: Default::default(),
                        }),
                        multisample: MultisampleState {
                            count: sample_count,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        label: None,
                    }),
            )
        }
    }

    impl Plane {
        /// constructor
        /// # Arguments
        /// - device: Device, provided by wgpu.
        /// - shader: the inputed fragment shader
        pub fn new(device: &Device, shader: &str) -> Plane {
            let module = create_module(device, shader).expect("Default shader is invalid");
            Plane {
                module,
                mouse: [0.0; 4],
                id: RenderID::gen(),
            }
        }

        pub fn set_shader(&mut self, device: &Device, shader: &str) {
            if let Some(module) = create_module(device, shader) {
                self.module = module;
            }
        }
    }

    fn create_module(device: &Device, shader: &str) -> Option<ShaderModule> {
        use naga::{front::wgsl::Parser, valid::*};
        let mut source = BASE_PREFIX.to_string();
        source += shader;
        source += BASE_SHADER;

        Validator::new(ValidationFlags::all(), Capabilities::empty())
            .validate(
                &Parser::new()
                    .parse(&source)
                    .map_err(|error| println!("WGSL Parse Error: {}", error))
                    .ok()?,
            )
            .map_err(|error| println!("WGSL Validation Error: {}", error))
            .ok()?;

        Some(device.create_shader_module(&ShaderModuleDescriptor {
            source: ShaderSource::Wgsl(source.into()),
            label: None,
        }))
    }
}
use plane::Plane;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut wb = winit::window::WindowBuilder::new();
    wb = wb.with_title("wGSL Sandbox");
    let window = wb.build(&event_loop).unwrap();
    let size = window.inner_size();
    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap()
    });

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Mailbox,
    };

    let mut surface = unsafe { instance.create_surface(&window) };
    surface.configure(&device, &config);
    // let mut swap_chain = device.create_swap_chain(&surface, &config);
    let handler = DeviceHandler::new(
        Arc::new(device),
        Arc::new(queue),
        Arc::new(Mutex::new(config)),
    );
    let mut scene = Scene::new(handler.clone(), &Default::default());
    let args: Vec<_> = std::env::args().collect();
    let source = if args.len() > 1 {
        match std::fs::read_to_string(&args[1]) {
            Ok(code) => code,
            Err(error) => {
                println!("{:?}", error);
                include_str!("newton-cuberoot.wgsl").to_string()
            }
        }
    } else {
        include_str!("newton-cuberoot.wgsl").to_string()
    };
    let mut plane = Plane::new(handler.device(), &source);
    // Adds a plane to the scene!
    scene.add_object(&mut plane);

    let mut dragging = false;
    let mut clicked = false;
    let mut cursor = [0.0; 2];
    event_loop.run(move |ev, _, control_flow| {
        *control_flow = match ev {
            Event::MainEventsCleared => {
                window.request_redraw();
                ControlFlow::Poll
            }
            Event::RedrawRequested(_) => {
                scene.update_bind_group(&plane);
                let frame = match surface.get_current_frame() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(handler.device().as_ref(), &handler.config());
                        surface
                            .get_current_frame()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let view = frame
                    .output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                scene.render_scene(&view);
                if clicked {
                    plane.mouse[3] = -plane.mouse[3];
                    clicked = false;
                }
                ControlFlow::Poll
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    let mut config = handler.lock_config().unwrap();
                    config.width = size.width;
                    config.height = size.height;
                    surface = unsafe { instance.create_surface(&window) };
                    ControlFlow::Poll
                }
                WindowEvent::CloseRequested => ControlFlow::Exit,
                WindowEvent::DroppedFile(path) => {
                    match std::fs::read_to_string(path) {
                        Ok(code) => {
                            plane.set_shader(handler.device(), &code);
                            scene.update_pipeline(&plane);
                        }
                        Err(error) => println!("{:?}", error),
                    }
                    ControlFlow::Poll
                }
                WindowEvent::MouseInput { state, .. } => {
                    dragging = state == ElementState::Pressed;
                    clicked = dragging;
                    if dragging {
                        plane.mouse[0] = cursor[0];
                        plane.mouse[1] = cursor[1];
                        plane.mouse[2] = cursor[0];
                        plane.mouse[3] = cursor[1];
                    } else {
                        plane.mouse[2] = -plane.mouse[2];
                    }
                    ControlFlow::Poll
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let height = scene.config().height as f32;
                    cursor = [position.x as f32, height - position.y as f32];
                    if dragging {
                        plane.mouse[0] = cursor[0];
                        plane.mouse[1] = cursor[1];
                    }
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            _ => ControlFlow::Poll,
        };
    })
}
