//! Visualization of shape and polygon mesh based on platform

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate truck_meshalgo;
extern crate truck_topology;
extern crate truck_platform;
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use std::sync::Arc;
use truck_platform::{wgpu::*, *};

/// Re-exports `truck_polymesh`.
pub mod polymesh {
    pub use truck_meshalgo::prelude::{base::*, PolygonMesh, StructuredMesh, Vertex};
}
pub use polymesh::*;

/// Material information.
///
/// Each instance is rendered based on the microfacet theory.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// albedo, base color, [0, 1]-normalized rgba. Default is `Vector4::new(1.0, 1.0, 1.0, 1.0)`.  
    /// Transparent by alpha is not yet supported in the current standard shader.
    pub albedo: Vector4,
    /// roughness of the surface: [0, 1]. Default is 0.5.
    pub roughness: f64,
    /// ratio of specular: [0, 1]. Default is 0.25.
    pub reflectance: f64,
    /// ratio of ambient: [0, 1]. Default is 0.02.
    pub ambient_ratio: f64,
    /// alpha blend flag
    pub alpha_blend: bool,
}

/// Configures of instances.
#[derive(Clone, Debug)]
pub struct InstanceState {
    /// instance matrix
    pub matrix: Matrix4,
    /// material of instance
    pub material: Material,
    /// texture of instance
    pub texture: Option<Arc<Texture>>,
    /// If this parameter is true, the backface culling will be activated.
    pub backface_culling: bool,
}

/// Configures of `WireFrameInstance`.
#[derive(Clone, Debug)]
pub struct WireFrameState {
    /// instance matrix
    pub matrix: Matrix4,
    /// color of instance
    pub color: Vector4,
}

/// Configures of polygon instance
#[derive(Clone, Debug, Default)]
pub struct PolygonInstanceDescriptor {
    /// configure of instance
    pub instance_state: InstanceState,
}

/// Configures of shape instance
#[derive(Clone, Debug)]
pub struct ShapeInstanceDescriptor {
    /// configure of instance
    pub instance_state: InstanceState,
    /// precision for meshing
    pub mesh_precision: f64,
}

/// Configures of wire frame instance of polygon
#[derive(Clone, Debug, Default)]
pub struct PolygonWireFrameDescriptor {
    /// configure of wire frame
    pub wireframe_state: WireFrameState,
}

/// Configures of wire frame instance of shape
#[derive(Clone, Debug)]
pub struct ShapeWireFrameDescriptor {
    /// configure of wire frame
    pub wireframe_state: WireFrameState,
    /// precision for polyline
    pub polyline_precision: f64,
}

/// shaders for rendering polygons
#[derive(Debug, Clone)]
pub struct PolygonShaders {
    vertex_module: Arc<ShaderModule>,
    vertex_entry: &'static str,
    fragment_module: Arc<ShaderModule>,
    fragment_entry: &'static str,
    tex_fragment_module: Arc<ShaderModule>,
    tex_fragment_entry: &'static str,
}

/// shaders for rendering wireframes
#[derive(Debug, Clone)]
pub struct WireShaders {
    vertex_module: Arc<ShaderModule>,
    vertex_entry: &'static str,
    fragment_module: Arc<ShaderModule>,
    fragment_entry: &'static str,
}

/// Instance of polygon
///
/// One can duplicate polygons with different postures and materials
/// that have the same mesh data.
/// To save memory, mesh data on the GPU can be used again.
///
/// The duplicated polygon by `Clone::clone` has the same mesh data and descriptor
/// with original, however, its render id is different from the one of original.
#[derive(Debug)]
pub struct PolygonInstance {
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    state: InstanceState,
    shaders: PolygonShaders,
    id: RenderID,
}

/// Wire frame rendering
#[derive(Debug)]
pub struct WireFrameInstance {
    vertices: Arc<BufferHandler>,
    strips: Arc<BufferHandler>,
    state: WireFrameState,
    shaders: WireShaders,
    id: RenderID,
}

/// Constroctor for instances
#[derive(Debug, Clone)]
pub struct InstanceCreator {
    handler: DeviceHandler,
    polygon_shaders: PolygonShaders,
    wire_shaders: WireShaders,
}

/// for creating `InstanceCreator`
pub trait CreatorCreator {
    /// create `InstanceCreator`
    fn instance_creator(&self) -> InstanceCreator;
}

/// The trait for Buffer Objects.
pub trait CreateBuffers {
    /// Creates buffer handlers of attributes and indices.
    fn buffers(
        &self,
        vertex_usage: BufferUsages,
        index_usage: BufferUsages,
        device: &Device,
    ) -> (BufferHandler, BufferHandler);
}

/// The trait for generating `Instance` from `Self`.
pub trait TryIntoInstance<I: Instance> {
    /// Configuation deacriptor for instance.
    type Descriptor;
    #[doc(hidden)]
    fn try_into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &I::Shaders,
        desc: &Self::Descriptor,
    ) -> Option<I>;
}

/// The trait for generating `Instance` from `Self`.
pub trait IntoInstance<I: Instance> {
    /// Configuation deacriptor for instance.
    type Descriptor;
    /// Creates `Instance` from `self`.
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &I::Shaders,
        desc: &Self::Descriptor,
    ) -> I;
}

/// Instance for rendering
pub trait Instance {
    #[doc(hidden)]
    type Shaders;
    /// Get standard shaders from instance creator.
    #[doc(hidden)]
    fn standard_shaders(creator: &InstanceCreator) -> Self::Shaders;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone)]
struct ExpandedPolygon<V> {
    vertices: Vec<V>,
    indices: Vec<u32>,
}

/// utility for creating `Texture`
pub mod image2texture;
mod instance_creator;
mod instance_descriptor;
mod polygon_instance;
mod polyrend;
mod shaperend;
mod wireframe_instance;
