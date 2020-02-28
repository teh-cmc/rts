use crate::{components, resources};
use cgmath::prelude::*;
use components::Vec2D;
// use raylib::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

enum SelectorState {
    Idle,
    Selecting(Entity, Vec2D),
    Confirmed(Entity, Vec2D, Vec2D),
}

pub struct Selector {
    state: SelectorState,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            state: SelectorState::Idle,
        }
    }
}

impl<'a> System<'a> for Selector {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, resources::Raylib>,
        ReadExpect<'a, resources::Camera>,
        ReadExpect<'a, resources::MouseState>,
        ReadExpect<'a, resources::BoundingTree>,
        ReadStorage<'a, components::Pos3D>,
        WriteStorage<'a, components::Pos2D>,
        WriteStorage<'a, components::Dim2D>,
        WriteStorage<'a, components::Selected>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (entities, mut rl, cam, mouse, bt, pos3Ds, mut pos2Ds, mut dim2Ds, mut selected) =
            sys_data;
        match self.state {
            SelectorState::Idle => {
                if mouse.is_pressed(0) {
                    let pos = mouse.position();
                    let e = entities
                        .build_entity()
                        .with(pos.into(), &mut pos2Ds)
                        .build();
                    self.state = SelectorState::Selecting(e, pos);
                }
            }
            SelectorState::Selecting(e, mut pos1) => {
                let mut pos2 = mouse.position();
                let mut dim = pos2 - pos1;
                if dim.x.is_negative() {
                    dim.x = dim.x.abs();
                    std::mem::swap(&mut pos1.x, &mut pos2.x);
                }
                if dim.y.is_negative() {
                    dim.y = dim.y.abs();
                    std::mem::swap(&mut pos1.y, &mut pos2.y);
                }

                *pos2Ds.get_mut(e).unwrap() = pos1.into();
                dim2Ds.insert(e, dim.into()).unwrap();

                if mouse.is_released(0) {
                    self.state = SelectorState::Confirmed(e, pos1, dim);
                }
            }
            SelectorState::Confirmed(e, pos, dim) => {
                entities.delete(e).unwrap();
                self.state = SelectorState::Idle;

                use raylib::prelude::RaylibMode3DExt;
                let mut proj = raylib::math::Matrix::identity();
                let mut mv = raylib::math::Matrix::identity();
                rl.draw(&unsafe { std::mem::transmute(()) }, |d| {
                    let _g = d.begin_mode_3D(cam.raw());

                    proj = hacks::get_matrix_projection();
                    mv = hacks::get_matrix_modelview();
                });

                let mat = mv * proj;
                let mat = mat.inverted();

                // use cgmath::{
                //     Matrix4 as CGMat4, Point3 as CGPnt3, Quaternion as CGQuat, SquareMatrix,
                //     Vector3 as CGVec3, Vector4 as CGVec4,
                // };
                // use collision::Ray3;
                // use raylib::core::math::Matrix as Lol;

                use raylib::core::math::{
                    Matrix as CGMat4, Quaternion as CGQuat, Ray as CGRay3, Vector3 as CGVec3,
                    Vector4 as CGVec4,
                };

                let (swidth, sheight) =
                    rl.read(|rl| (rl.get_screen_width() as f32, rl.get_screen_height() as f32));

                const UPPER_LEFT: usize = 0;
                const UPPER_RIGHT: usize = 1;
                const BOTTOM_LEFT: usize = 2;
                const BOTTOM_RIGHT: usize = 3;
                let clicks = &[
                    (pos.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32 + dim.y as f32),
                    (pos.x as f32, pos.y as f32 + dim.y as f32),
                ];

                let mut rays = Vec::with_capacity(4);
                for pos in clicks {
                    let (x, y) = ((2. * pos.0) / swidth - 1., 1. - (2. * pos.1) / sheight);

                    let near = {
                        let quat = CGQuat::new(x, y, 0., 1.);
                        let quat = quat.transform(mat);
                        CGVec3::new(quat.x / quat.w, quat.y / quat.w, quat.z / quat.w)
                    };
                    let far = {
                        let quat = CGQuat::new(x, y, 1., 1.);
                        let quat = quat.transform(mat);
                        CGVec3::new(quat.x / quat.w, quat.y / quat.w, quat.z / quat.w)
                    };

                    let xxx = far - near;
                    // let r1 = CGRay3 {
                    //     position: cam.raw().position,
                    //     direction: xxx,
                    // };

                    let r1 = collision::Ray3::new(
                        cgmath::Point3::new(near.x, near.y, near.z),
                        cgmath::Vector3::new(xxx.x, xxx.y, xxx.z).normalize(),
                    );

                    // rays.push(r1);

                    rays.push((
                        cgmath::Point3::new(far.x, far.y, far.z),
                        cgmath::Point3::new(near.x, near.y, near.z),
                    ));
                }

                use collision::{Frustum, Plane};
                dbg!(&rays);
                let frustum = Frustum::new(
                    /* lft */
                    Plane::from_points(rays[0].0, rays[0].1, rays[3].0).unwrap(),
                    /* rgt */
                    Plane::from_points(rays[1].0, rays[1].1, rays[2].0).unwrap(),
                    /* btm */
                    Plane::from_points(rays[3].0, rays[3].1, rays[2].0).unwrap(),
                    /* top */
                    Plane::from_points(rays[0].0, rays[0].1, rays[1].0).unwrap(),
                    /* nar */
                    Plane::from_points(rays[0].0, rays[1].0, rays[2].0).unwrap(),
                    /* far */
                    Plane::from_points(rays[0].1, rays[1].1, rays[2].1).unwrap(),
                );

                selected.clear();
                dbg!(frustum);
                for e in bt.test_frustum(&frustum).collect::<Vec<_>>() {
                    selected.insert(e, components::Selected).unwrap();
                }
                // for r in rays {
                //     for e in bt.test_ray(&r).collect::<Vec<_>>() {
                //         selected.insert(e, components::Selected).unwrap();
                //     }
                // }
            }
        }
    }
}

// -----------------------------------------------------------------------------

mod hacks {
    use raylib::{core::math::Matrix, ffi::Matrix as c_matrix};

    extern "C" {
        fn GetMatrixProjection() -> c_matrix;
        fn GetMatrixModelview() -> c_matrix;
    }

    pub fn get_matrix_projection() -> Matrix {
        unsafe { GetMatrixProjection().into() }
    }
    pub fn get_matrix_modelview() -> Matrix {
        unsafe { GetMatrixModelview().into() }
    }
}
