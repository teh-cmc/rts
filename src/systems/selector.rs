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
        ReadExpect<'a, resources::Raylib>,
        ReadExpect<'a, resources::Camera>,
        ReadExpect<'a, resources::MouseState>,
        ReadExpect<'a, resources::BoundingTree>,
        ReadStorage<'a, components::Pos3D>,
        WriteStorage<'a, components::Pos2D>,
        WriteStorage<'a, components::Dim2D>,
        WriteStorage<'a, components::Selected>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (entities, rl, cam, mouse, bt, pos3Ds, mut pos2Ds, mut dim2Ds, mut selected) = sys_data;
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

                use cgmath::{Point3, Vector3};
                use collision::{Frustum, Plane, Ray3};
                use raylib::core::math::Vector2;

                let pos = Vector2::new(pos.x as f32, pos.y as f32);
                dbg!(dim);

                let pos1 = Vector2::new(pos.x as f32, pos.y as f32);
                let r1 = rl.read(|rl| rl.get_mouse_ray(pos1, cam.raw()));
                let mut r1 = Ray3::new(
                    (r1.position.x, r1.position.y, r1.position.z).into(),
                    (r1.direction.x, r1.direction.y, r1.direction.z).into(),
                );
                let mut r1far = r1;
                r1far.origin = r1.origin + (r1.direction.normalize() * 1000.0);
                r1.origin = r1.origin + (r1.direction.normalize() * 0.01);

                let pos2 = Vector2::new(pos.x as f32 + dim.x as f32, pos.y as f32);
                let r2 = rl.read(|rl| rl.get_mouse_ray(pos2, cam.raw()));
                let mut r2 = Ray3::new(
                    (r2.position.x, r2.position.y, r2.position.z).into(),
                    (r2.direction.x, r2.direction.y, r2.direction.z).into(),
                );
                let mut r2far = r2;
                r2far.origin = r2.origin + (r2.direction.normalize() * 1000.0);
                r2.origin = r2.origin + (r2.direction.normalize() * 0.01);

                let pos3 = Vector2::new(pos.x as f32 + dim.x as f32, pos.y as f32 + dim.y as f32);
                let r3 = rl.read(|rl| rl.get_mouse_ray(pos3, cam.raw()));
                let mut r3 = Ray3::new(
                    (r3.position.x, r3.position.y, r3.position.z).into(),
                    (r3.direction.x, r3.direction.y, r3.direction.z).into(),
                );
                let mut r3far = r3;
                r3far.origin = r3.origin + (r3.direction.normalize() * 1000.0);
                r3.origin = r3.origin + (r3.direction.normalize() * 0.01);

                let pos4 = Vector2::new(pos.x as f32, pos.y as f32 + dim.y as f32);
                let r4 = rl.read(|rl| rl.get_mouse_ray(pos4, cam.raw()));
                let mut r4 = Ray3::new(
                    (r4.position.x, r4.position.y, r4.position.z).into(),
                    (r4.direction.x, r4.direction.y, r4.direction.z).into(),
                );
                let mut r4far = r4;
                r4far.origin = r4.origin + (r4.direction.normalize() * 1000.0);
                r4.origin = r4.origin + (r4.direction.normalize() * 0.01);

                if pos1 != pos3 {
                    dbg!(pos1, pos2, pos3, pos4);

                    let pnear = Plane::from_points(r1.origin, r2.origin, r3.origin).unwrap();
                    let pfar = {
                        let mut pfar = pnear;
                        pfar.d -= 1000.0;
                        pfar
                    };
                    // let pfar = Plane::from_points(r1far.origin, r2far.origin,
                    // r3far.origin).unwrap();
                    let pleft = Plane::from_points(r1.origin, r4.origin, r1far.origin).unwrap();
                    let pright = {
                        let mut pright = pleft;
                        pright.d -= 1000.0;
                        pright
                    };
                    // let pright = Plane::from_points(r2.origin, r3.origin, r2far.origin).unwrap();
                    let pbottom = Plane::from_points(r3.origin, r4.origin, r3far.origin).unwrap();
                    let ptop = {
                        let mut ptop = pbottom;
                        ptop.d += 1000.0;
                        ptop
                    };
                    // let ptop = Plane::from_points(r1.origin, r2.origin, r1far.origin).unwrap();

                    let frust = Frustum::new(pleft, pright, pbottom, ptop, pnear, pfar);
                    dbg!(frust);
                    dbg!(bt.test_frustrum(&frust).collect::<Vec<_>>());
                }

                selected.clear();
                for e in dbg!(bt.test_ray(&r1).collect::<Vec<_>>()) {
                    selected.insert(e, components::Selected).unwrap();
                }
                for e in dbg!(bt.test_ray(&r2).collect::<Vec<_>>()) {
                    selected.insert(e, components::Selected).unwrap();
                }
                for e in dbg!(bt.test_ray(&r3).collect::<Vec<_>>()) {
                    selected.insert(e, components::Selected).unwrap();
                }
                for e in dbg!(bt.test_ray(&r4).collect::<Vec<_>>()) {
                    selected.insert(e, components::Selected).unwrap();
                }
            }
        }
    }
}
