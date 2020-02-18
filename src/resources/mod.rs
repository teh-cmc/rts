mod camera;
mod mouse;
mod raylib;

pub use self::{camera::Camera, mouse::MouseState, raylib::Raylib};

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct DeltaTime(pub f32);
