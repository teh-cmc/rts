use cgmath::{prelude::*, Vector2, Vector3};
use specs::{prelude::*, storage::HashMapStorage};

// -----------------------------------------------------------------------------

pub type Vec2D = Vector2<i32>;

#[derive(Clone, Copy, Debug)]
pub struct Pos2D(pub Vec2D);
impl Component for Pos2D {
    type Storage = HashMapStorage<Self>;
}

impl From<Vec2D> for Pos2D {
    fn from(pos: Vec2D) -> Self {
        Self(pos)
    }
}

#[derive(Default, Debug)]
pub struct Pos2DInvalidated;
impl Component for Pos2DInvalidated {
    type Storage = NullStorage<Self>;
}

#[derive(Clone, Copy, Debug)]
pub struct Dim2D(pub Vec2D);
impl Component for Dim2D {
    type Storage = HashMapStorage<Self>;
}

impl From<Vec2D> for Dim2D {
    fn from(dim: Vec2D) -> Self {
        Self(dim)
    }
}

#[derive(Default, Debug)]
pub struct Dim2DInvalidated;
impl Component for Dim2DInvalidated {
    type Storage = NullStorage<Self>;
}

// TODO(cmc): kill these?
use raylib::core::math::Vector2 as RayVector2;
impl Into<RayVector2> for Pos2D {
    fn into(self) -> RayVector2 {
        RayVector2::new(self.0.x as f32, self.0.y as f32)
    }
}
impl Into<RayVector2> for &Pos2D {
    fn into(self) -> RayVector2 {
        RayVector2::new(self.0.x as f32, self.0.y as f32)
    }
}
impl Into<RayVector2> for Dim2D {
    fn into(self) -> RayVector2 {
        RayVector2::new(self.0.x as f32, self.0.y as f32)
    }
}
impl Into<RayVector2> for &Dim2D {
    fn into(self) -> RayVector2 {
        RayVector2::new(self.0.x as f32, self.0.y as f32)
    }
}

// -----------------------------------------------------------------------------

pub type Vec3D = Vector3<f32>;

#[derive(Clone, Copy, Debug)]
pub struct Pos3D(pub Vec3D);
impl Component for Pos3D {
    type Storage = VecStorage<Self>;
}

impl From<Vec3D> for Pos3D {
    fn from(pos: Vec3D) -> Self {
        Self(pos)
    }
}

#[derive(Default, Debug)]
pub struct Pos3DInvalidated;
impl Component for Pos3DInvalidated {
    type Storage = NullStorage<Self>;
}

#[derive(Clone, Copy, Debug)]
pub struct Dim3D(pub Vec3D);
impl Component for Dim3D {
    type Storage = VecStorage<Self>;
}

impl From<Vec3D> for Dim3D {
    fn from(dim: Vec3D) -> Self {
        Self(dim)
    }
}

#[derive(Default, Debug)]
pub struct Dim3DInvalidated;
impl Component for Dim3DInvalidated {
    type Storage = NullStorage<Self>;
}

// TODO(cmc): kill these?
use raylib::core::math::Vector3 as RayVector3;
impl Into<RayVector3> for Pos3D {
    fn into(self) -> RayVector3 {
        RayVector3::new(self.0.x, self.0.y, self.0.z)
    }
}
impl Into<RayVector3> for &Pos3D {
    fn into(self) -> RayVector3 {
        RayVector3::new(self.0.x, self.0.y, self.0.z)
    }
}
impl Into<RayVector3> for Dim3D {
    fn into(self) -> RayVector3 {
        RayVector3::new(self.0.x, self.0.y, self.0.z)
    }
}
impl Into<RayVector3> for &Dim3D {
    fn into(self) -> RayVector3 {
        RayVector3::new(self.0.x, self.0.y, self.0.z)
    }
}

// -----------------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct Selected;
impl Component for Selected {
    type Storage = NullStorage<Self>;
}
