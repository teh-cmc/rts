use crate::maths::{Point3, Vec2, Vec2i, Vec3};
use specs::{prelude::*, storage::HashMapStorage, Component};

pub mod prelude {
    pub use super::{
        Color as CompColor, Mesh as CompMesh, Pos2D as CompPos2D,
        Pos2DInvalidated as CompPos2DInvalidated, Pos3D as CompPos3D,
        Pos3DInvalidated as CompPos3DInvalidated, Selected as CompSelected,
    };
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Component)]
#[storage(HashMapStorage)]
pub struct Pos2D(pub Vec2i);

#[derive(Clone, Copy, Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Pos2DInvalidated;

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub struct Pos3D(pub Vec3);

#[derive(Clone, Copy, Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Pos3DInvalidated;

#[derive(Clone, Copy, Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Selected;

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub struct Color(pub raylib::color::Color);

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub enum Mesh {
    Rect { dimensions: Vec2i },
    Cube { dimensions: Vec3 },
    Line { a: Point3, b: Point3 },
}
