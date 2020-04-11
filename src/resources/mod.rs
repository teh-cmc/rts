mod bounding_tree;
mod camera;
mod models;
mod mouse;
mod raylib;

pub mod prelude {
    pub use super::{
        bounding_tree::BoundingTree as ResrcBoundingTree,
        camera::{Camera as ResrcCamera, Mode as ResrcCameraMode},
        models::{MeshID, MeshStore as ResrcMeshStore, Model},
        mouse::MouseState as ResrcMouseState,
        raylib::Raylib as ResrcRaylib,
        DeltaTime as ResrcDeltaTime, ModelView as ResrcModelView, Projection as ResrcProjection,
    };
}

// -----------------------------------------------------------------------------

use crate::maths::Mat4;

#[derive(Default)]
pub struct DeltaTime(pub f32);

#[derive(Default)]
pub struct ModelView(pub Mat4);

#[derive(Default)]
pub struct Projection(pub Mat4);
