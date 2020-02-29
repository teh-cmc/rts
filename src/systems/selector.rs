use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use raylib::color::Color;
use specs::prelude::*;

// -----------------------------------------------------------------------------

enum SelectorState {
    Idle,
    Selecting(Entity, Vec2i),
    Confirmed(Entity, Vec2i, Vec2i),
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

// TODO(cmc): struct systemdata
impl<'a> System<'a> for Selector {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, ResrcRaylib>,
        ReadExpect<'a, ResrcMouseState>,
        ReadExpect<'a, ResrcBoundingTree>,
        ReadExpect<'a, ResrcProjection>,
        ReadExpect<'a, ResrcModelView>,
        WriteStorage<'a, CompPos3D>,
        WriteStorage<'a, CompPos2D>,
        WriteStorage<'a, CompMesh>,
        WriteStorage<'a, CompSelected>,
        WriteStorage<'a, CompColor>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            rl,
            mouse,
            bt,
            m_proj,
            m_view,
            mut pos3Ds,
            mut pos2Ds,
            mut meshes,
            mut selected,
            mut colors,
        ) = sys_data;
        match self.state {
            SelectorState::Idle => {
                if mouse.is_pressed(0) {
                    let pos = mouse.position();
                    let e = entities
                        .build_entity()
                        .with(CompPos2D(pos), &mut pos2Ds)
                        .build();
                    self.state = SelectorState::Selecting(e, pos);
                }
            }
            SelectorState::Selecting(e, mut pos1) => {
                let mut pos2 = mouse.position();
                let mut dim = *pos2 - *pos1;
                if dim.x.is_negative() {
                    dim.x = dim.x.abs();
                    std::mem::swap(&mut pos1.x, &mut pos2.x);
                }
                if dim.y.is_negative() {
                    dim.y = dim.y.abs();
                    std::mem::swap(&mut pos1.y, &mut pos2.y);
                }

                *pos2Ds.get_mut(e).unwrap() = CompPos2D(pos1);
                let rect = CompMesh::Rect {
                    dimensions: dim.into(),
                };
                let color = CompColor(Color::GREEN);
                meshes.insert(e, rect).unwrap();
                colors.insert(e, color).unwrap();

                if mouse.is_released(0) {
                    self.state = SelectorState::Confirmed(e, pos1, dim.into());
                }
            }
            SelectorState::Confirmed(e, pos, dim) => {
                entities.delete(e).unwrap();
                self.state = SelectorState::Idle;

                let mat = *m_view.0 * *m_proj.0;
                let mat = mat.invert().unwrap();
                let (swidth, sheight) =
                    rl.read(|rl| (rl.get_screen_width() as f32, rl.get_screen_height() as f32));

                let corners = &[
                    (pos.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32 + dim.y as f32),
                    (pos.x as f32, pos.y as f32 + dim.y as f32),
                ];

                let corners = corners.into_iter().map(|pos| {
                    let (x, y) = ((2. * pos.0) / swidth - 1., 1. - (2. * pos.1) / sheight);

                    let near: Point3 = {
                        let pos: Vec4 = (x, y, 0., 1.).into();
                        let pos = dbg!(mat) * dbg!(*pos);
                        (pos.x / pos.w, pos.y / pos.w, pos.z / pos.w).into()
                    };
                    let far: Point3 = {
                        let pos: Vec4 = (x, y, 1., 1.).into();
                        let pos = mat * *pos;
                        (pos.x / pos.w, pos.y / pos.w, pos.z / pos.w).into()
                    };

                    (near, far)
                });

                selected.clear();
                corners
                    .map(|(near, far)| {
                        let line = CompMesh::Line { a: near, b: far };
                        let color = CompColor(Color::DARKBLUE);
                        let _ = entities
                            .build_entity()
                            .with(CompPos3D(near.into()), &mut pos3Ds)
                            .with(line, &mut meshes)
                            .with(color, &mut colors)
                            .build();

                        let dir = (*far - *near).normalize();
                        collision::Ray3::new(*near, dir)
                    })
                    .for_each(|r| {
                        for e in bt.test_ray(&r).collect::<Vec<_>>() {
                            selected.insert(e, CompSelected).unwrap();
                        }
                    });

                // const UPPER_LEFT: usize = 0;
                // const UPPER_RIGHT: usize = 1;
                // const BOTTOM_LEFT: usize = 2;
                // const BOTTOM_RIGHT: usize = 3;

                // use collision::{Frustum, Plane};
                // dbg!(&rays);
                // let frustum = Frustum::new(
                //     /* lft */
                //     Plane::from_points(rays[0].0, rays[0].1,
                // rays[3].0).unwrap(),     /* rgt */
                //     Plane::from_points(rays[1].0, rays[1].1,
                // rays[2].0).unwrap(),     /* btm */
                //     Plane::from_points(rays[3].0, rays[3].1,
                // rays[2].0).unwrap(),     /* top */
                //     Plane::from_points(rays[0].0, rays[0].1,
                // rays[1].0).unwrap(),     /* nar */
                //     Plane::from_points(rays[0].0, rays[1].0,
                // rays[2].0).unwrap(),     /* far */
                //     Plane::from_points(rays[0].1, rays[1].1,
                // rays[2].1).unwrap(), );

                // selected.clear();
                // dbg!(frustum);
                // for e in bt.test_frustum(&frustum).collect::<Vec<_>>() {
                //     selected.insert(e, CompSelected).unwrap();
                // }
            }
        }
    }
}
