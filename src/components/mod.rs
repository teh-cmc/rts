use raylib::prelude::*;
use specs::{prelude::*, storage::HashMapStorage};

// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Pos3D(pub Vector3);
impl Component for Pos3D {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Copy, Debug)]
pub struct Pos2D(pub i32, pub i32);
impl Component for Pos2D {
    type Storage = HashMapStorage<Self>;
}
impl From<(i32, i32)> for Pos2D {
    fn from((x, y): (i32, i32)) -> Self {
        Self(x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dim2D(pub i32, pub i32);
impl Component for Dim2D {
    type Storage = HashMapStorage<Self>;
}
impl From<(i32, i32)> for Dim2D {
    fn from((x, y): (i32, i32)) -> Self {
        Self(x, y)
    }
}

#[derive(Default, Debug)]
pub struct Selected;
impl Component for Selected {
    type Storage = NullStorage<Self>;
}
