use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
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
        ReadExpect<'a, ResrcMeshStore>,
        WriteStorage<'a, CompDirectShape>,
        WriteStorage<'a, CompModel3D>,
        WriteStorage<'a, CompTransform3D>,
        WriteStorage<'a, CompSelected>,
        WriteStorage<'a, CompColor>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            entities,
            mut rl,
            mouse,
            bt,
            m_proj,
            m_view,
            meshes,
            mut shapes,
            mut models,
            mut transforms,
            mut selected,
            mut colors,
        ) = sys_data;

        match self.state {
            SelectorState::Idle => {
                if mouse.is_pressed(0) {
                    let pos = mouse.position();
                    let e = entities.build_entity().build();
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

                let rect = CompDirectShape::Rect {
                    pos: pos1,
                    dimensions: dim.into(),
                };
                let color = CompColor(Color::GREEN);
                shapes.insert(e, rect).unwrap();
                colors.insert(e, color).unwrap();

                if mouse.is_released(0) {
                    self.state = SelectorState::Confirmed(e, pos1, dim.into());
                }
            }
            SelectorState::Confirmed(e, pos, dim) => {
                entities.delete(e).unwrap();
                self.state = SelectorState::Idle;

                let mat = *m_proj.0 * *m_view.0;
                let mat = mat.invert().unwrap();
                let (swidth, sheight) =
                    rl.read(|rl| (rl.get_screen_width() as f32, rl.get_screen_height() as f32));

                let corners = &[
                    (pos.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32),
                    (pos.x as f32 + dim.x as f32, pos.y as f32 + dim.y as f32),
                    (pos.x as f32, pos.y as f32 + dim.y as f32),
                ];

                let corners: Vec<_> = corners
                    .into_iter()
                    .map(|pos| {
                        let (x, y) = ((2. * pos.0) / swidth - 1., 1. - (2. * pos.1) / sheight);

                        let near: Point3 = {
                            let pos: Vec4 = (x, y, 0., 1.).into();
                            let pos = mat * *pos;
                            (pos.x / pos.w, pos.y / pos.w, pos.z / pos.w).into()
                        };
                        let far: Point3 = {
                            let pos: Vec4 = (x, y, 1., 1.).into();
                            let pos = mat * *pos;
                            (pos.x / pos.w, pos.y / pos.w, pos.z / pos.w).into()
                        };

                        (near, far)
                    })
                    .collect();

                selected.clear();
                corners
                    .iter()
                    .map(|&(near, far)| {
                        let dir = (*far - *near).normalize();
                        collision::Ray3::new(*near, dir)
                    })
                    .for_each(|r| {
                        for e in bt.test_ray(&r).collect::<Vec<_>>() {
                            selected.insert(e, CompSelected).unwrap();
                        }
                    });

                use collision::{Frustum, Plane};
                let planes = vec![
                    /* lft */ (corners[0].0, corners[0].1, corners[3].1, corners[3].0),
                    /* rgt */ (corners[1].0, corners[1].1, corners[2].1, corners[2].0),
                    /* btm */ (corners[3].0, corners[3].1, corners[2].1, corners[2].0),
                    /* top */ (corners[0].0, corners[0].1, corners[1].1, corners[1].0),
                    /* nar */ (corners[0].0, corners[1].0, corners[2].0, corners[3].0),
                    /* far */ (corners[0].1, corners[1].1, corners[2].1, corners[3].1),
                ];

                for p in &planes {
                    let wf = CompDirectShape::WireFrame {
                        vertices: vec![p.0, p.1, p.2, p.3],
                    };
                    let color = CompColor(Color::BLACK);

                    let _ = entities
                        .build_entity()
                        .with(wf, &mut shapes)
                        .with(color, &mut colors)
                        .build();
                }

                let frustum = (|| {
                    macro_rules! plane {
                        ($p1:expr, $p2:expr, $p3:expr) => {
                            Plane::from_points($p1, $p2, $p3).ok_or(anyhow!("illegal plane"))
                        };
                    }
                    let f = Frustum::new(
                        /* lft */
                        plane!(*planes[0].0, *planes[0].1, *planes[0].2)?,
                        /* rgt */
                        plane!(*planes[1].0, *planes[1].1, *planes[1].2)?,
                        /* btm */
                        plane!(*planes[2].0, *planes[2].1, *planes[2].2)?,
                        /* top */
                        plane!(*planes[3].0, *planes[3].1, *planes[3].2)?,
                        /* nar */
                        plane!(*planes[4].0, *planes[4].1, *planes[4].2)?,
                        /* far */
                        plane!(*planes[5].0, *planes[5].1, *planes[5].2)?,
                    );
                    Ok::<_, AnyError>(f)
                })();

                if frustum.is_err() {
                    return;
                }
                let frustum = frustum.unwrap();

                // selected.clear();
                dbg!(frustum);
                use cgmath::Rad;
                use collision::{Aabb3, Projection, Relation, Sphere};
                dbg!(frustum.contains(&Sphere {
                    center: (0f32, 0f32, 0f32).into(),
                    radius: 1f32,
                }));
                dbg!(frustum.contains(&Aabb3::new((0., 0., 0.).into(), (10., 10., 10.).into())));
                for e in bt.test_frustum(&frustum).collect::<Vec<_>>() {
                    selected.insert(e, CompSelected).unwrap();
                }
            }
        }
    }
}
