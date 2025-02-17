//! Simple OBJ viewer
//!
//! - Drag the mouse to rotate the model.
//! - Drag and drop obj files into the window to switch models.
//! - Right-click to move the light to the camera's position.
//! - Enter "P" on the keyboard to switch between parallel projection and perspective projection of the camera.
//! - Enter "L" on the keyboard to switch the point light source/uniform light source of the light.
//! - Enter "Space" on the keyboard to switch the rendering mode for the wireframe and surface.

use std::io::Read;
use truck_meshalgo::prelude::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;

enum RenderMode {
    NaiveSurface,
    NaiveWireFrame,
    HiddenLineEliminate,
    SurfaceAndWireFrame,
}

struct MyApp {
    scene: Scene,
    creator: InstanceCreator,
    rotate_flag: bool,
    prev_cursor: Vector2,
    instance: PolygonInstance,
    wireframe: WireFrameInstance,
    render_mode: RenderMode,
}

impl MyApp {
    fn create_camera() -> Camera {
        let matrix = Matrix4::look_at_rh(
            Point3::new(1.0, 1.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            matrix.invert().unwrap(),
            Rad(std::f64::consts::PI / 4.0),
            0.1,
            40.0,
        )
    }

    fn update_render_mode(&mut self) {
        self.scene.clear_objects();
        match self.render_mode {
            RenderMode::NaiveSurface => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    alpha_blend: false,
                };
                self.scene.add_object(&self.instance);
            }
            RenderMode::NaiveWireFrame => {
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                self.scene.add_object(&self.wireframe);
            }
            RenderMode::HiddenLineEliminate => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(0.0, 0.0, 0.0, 1.0),
                    reflectance: 0.0,
                    roughness: 0.0,
                    ambient_ratio: 1.0,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                self.scene.add_object(&self.instance);
                self.scene.add_object(&self.wireframe);
            }
            RenderMode::SurfaceAndWireFrame => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(0.0, 0.0, 0.0, 1.0);
                self.scene.add_object(&self.instance);
                self.scene.add_object(&self.wireframe);
            }
        }
    }

    fn load_obj<R: Read>(
        creator: &InstanceCreator,
        reader: R,
    ) -> (PolygonInstance, WireFrameInstance) {
        let mut mesh = obj::read(reader).unwrap();
        mesh.put_together_same_attrs()
            .add_smooth_normals(0.5, false);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let mat = Matrix4::from_translation(center.to_vec()) * Matrix4::from_scale(size);
        let inst_desc = PolygonInstanceDescriptor {
            instance_state: InstanceState {
                matrix: mat.invert().unwrap(),
                ..Default::default()
            },
            ..Default::default()
        };
        let wire_inst_desc = PolygonWireFrameDescriptor {
            wireframe_state: WireFrameState {
                matrix: mat.invert().unwrap(),
                ..Default::default()
            },
            ..Default::default()
        };
        (
            creator.create_instance(&mesh, &inst_desc),
            creator.create_instance(&mesh, &wire_inst_desc),
        )
    }
}

impl App for MyApp {
    fn init(handler: &DeviceHandler, info: AdapterInfo) -> MyApp {
        let sample_count = match info.backend {
            Backend::Vulkan => 2,
            Backend::Dx12 => 2,
            _ => 1,
        };
        let scene_desc = SceneDescriptor {
            background: Color::BLACK,
            camera: MyApp::create_camera(),
            lights: vec![Light {
                position: Point3::new(1.0, 1.0, 1.0),
                color: Vector3::new(1.0, 1.0, 1.0),
                light_type: LightType::Point,
            }],
            sample_count,
        };
        let scene = Scene::new(handler.clone(), &scene_desc);
        let creator = scene.instance_creator();
        let (instance, wireframe) =
            MyApp::load_obj(&creator, include_bytes!("teapot.obj").as_ref());
        let mut app = MyApp {
            scene,
            creator,
            rotate_flag: false,
            prev_cursor: Vector2::zero(),
            instance,
            wireframe,
            render_mode: RenderMode::NaiveSurface,
        };
        app.update_render_mode();
        app
    }

    fn app_title<'a>() -> Option<&'a str> { Some("simple obj viewer") }

    fn dropped_file(&mut self, path: std::path::PathBuf) -> ControlFlow {
        let file = std::fs::File::open(path).unwrap();
        let (instance, wireframe) = MyApp::load_obj(&self.creator, file);
        self.instance = instance;
        self.wireframe = wireframe;
        self.update_render_mode();
        Self::default_control_flow()
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
            }
            MouseButton::Right => {
                let (light, camera) = {
                    let desc = self.scene.descriptor_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                match light.light_type {
                    LightType::Point => {
                        light.position = camera.position();
                    }
                    LightType::Uniform => {
                        light.position = camera.position();
                        let strength = light.position.to_vec().magnitude();
                        light.position /= strength;
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _: TouchPhase) -> ControlFlow {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let camera = &mut self.scene.descriptor_mut().camera;
                let trans_vec = camera.eye_direction() * 0.2 * y as f64;
                camera.matrix = Matrix4::from_translation(trans_vec) * camera.matrix;
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        let position = Vector2::new(position.x, position.y);
        if self.rotate_flag {
            let matrix = &mut self.scene.descriptor_mut().camera.matrix;
            let position = Vector2::new(position.x, position.y);
            let dir2d = &position - self.prev_cursor;
            if dir2d.so_small() {
                return Self::default_control_flow();
            }
            let mut axis = dir2d[1] * matrix[0].truncate();
            axis += dir2d[0] * &matrix[1].truncate();
            axis /= axis.magnitude();
            let angle = dir2d.magnitude() * 0.01;
            let mat = Matrix4::from_axis_angle(axis, Rad(angle));
            *matrix = mat.invert().unwrap() * *matrix;
        }
        self.prev_cursor = position;
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, input: KeyboardInput, _: bool) -> ControlFlow {
        if input.state == ElementState::Released {
            return Self::default_control_flow();
        }
        let keycode = match input.virtual_keycode {
            Some(keycode) => keycode,
            None => return Self::default_control_flow(),
        };
        match keycode {
            VirtualKeyCode::P => {
                let camera = &mut self.scene.descriptor_mut().camera;
                *camera = match camera.projection_type() {
                    ProjectionType::Parallel => Camera::perspective_camera(
                        camera.matrix,
                        Rad(std::f64::consts::PI / 4.0),
                        0.1,
                        40.0,
                    ),
                    ProjectionType::Perspective => {
                        Camera::parallel_camera(camera.matrix, 1.0, 0.1, 40.0)
                    }
                };
            }
            VirtualKeyCode::L => {
                let (light, camera) = {
                    let desc = self.scene.descriptor_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                *light = match light.light_type {
                    LightType::Point => {
                        let mut vec = camera.position();
                        vec /= vec.to_vec().magnitude();
                        Light {
                            position: vec,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = camera.position();
                        Light {
                            position,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Point,
                        }
                    }
                };
            }
            VirtualKeyCode::Space => {
                self.render_mode = match self.render_mode {
                    RenderMode::NaiveSurface => RenderMode::SurfaceAndWireFrame,
                    RenderMode::SurfaceAndWireFrame => RenderMode::NaiveWireFrame,
                    RenderMode::NaiveWireFrame => RenderMode::HiddenLineEliminate,
                    RenderMode::HiddenLineEliminate => RenderMode::NaiveSurface,
                };
                self.update_render_mode();
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn render(&mut self, view: &TextureView) { self.scene.render_scene(view); }
}

fn main() { MyApp::run(); }
