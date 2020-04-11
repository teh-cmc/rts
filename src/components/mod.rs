use crate::{
    maths::{Mat4, Point3, Vec2, Vec2i, Vec3, Vec3i},
    resources::prelude::Model,
    voxel::VoxelModel as RawVoxelModel,
};
use specs::{prelude::*, storage::HashMapStorage, Component};
use std::sync::Arc;

pub mod prelude {
    pub use super::{
        Color as CompColor, DirectShape as CompDirectShape, GridPosition as CompGridPosition,
        Invalidated as CompInvalidated, Model3D as CompModel3D, Selected as CompSelected,
        Transform3D as CompTransform3D, VoxelModel as CompVoxelModel,
    };
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Invalidated;

#[derive(Clone, Copy, Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Selected;

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub struct Color(pub raylib::color::Color);

#[derive(Clone, Debug, Component)]
#[storage(HashMapStorage)]
pub enum DirectShape {
    Rect { pos: Vec2i, dimensions: Vec2i },
    WireFrame { vertices: Vec<Point3> },
}

#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct VoxelModel(pub RawVoxelModel);

#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct GridPosition(pub Vec3i);

// -----------------------------------------------------------------------------

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Model3D(pub Arc<Model>);

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub struct Transform3D(pub Mat4);
