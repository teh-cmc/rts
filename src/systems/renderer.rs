use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use raylib::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Renderer(Option<RaylibThread>);

impl Renderer {
    pub fn new(thread: RaylibThread) -> Self {
        Self(thread.into())
    }
}

// TODO(cmc): systemdata struct
impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, ResrcRaylib>,
        WriteExpect<'a, ResrcModelView>,
        WriteExpect<'a, ResrcProjection>,
        ReadExpect<'a, ResrcCamera>,
        Entities<'a>,
        ReadStorage<'a, CompDirectShape>,
        WriteStorage<'a, CompModel3D>,
        ReadStorage<'a, CompTransform3D>,
        ReadStorage<'a, CompSelected>,
        ReadStorage<'a, CompColor>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (
            mut rl,
            mut m_view,
            mut m_proj,
            cam,
            entities,
            shapes,
            mut models,
            transforms,
            selected,
            colors,
        ) = sys_data;

        let thread = self.0.as_ref().unwrap();
        let (swidth, sheight) = (rl.read(|rl| rl.get_screen_width() - 100), 40);
        let (x, y) = rl.read(|rl| (rl.get_mouse_x() as f32, rl.get_mouse_y() as f32));
        rl.draw(&thread, |d| {
            d.clear_background(Color::DARKGRAY);

            {
                let mut d2 = d.begin_mode_3D(cam.raw());

                // Projection and ModelView matrices corresponding to the main
                // camera have now been loaded on the other side of the FFI
                // barrier: grab 'em!
                *m_view.0 = *hacks::get_matrix_modelview();
                *m_proj.0 = *hacks::get_matrix_projection();

                for (
                    e,
                    &mut CompModel3D(ref mut model),
                    &CompTransform3D(transform),
                    &CompColor(color),
                ) in (&entities, &mut models, &transforms, &colors).join()
                {
                    // TODO(cmc): something smarter
                    let color = if let Some(_) = selected.get(e) {
                        Color::GOLD
                    } else {
                        color
                    };

                    // d2.draw_cube(
                    //     Vector3::new(transform.w.x, transform.w.y, transform.w.z),
                    //     2.,
                    //     2.,
                    //     2.,
                    //     Color::RED,
                    // );

                    // TODO(cmc): needs interior mutability... or a transform
                    // dedicated system?
                    model.set_transform(&transform.into());
                    d2.draw_model(model, Vector3::new(0., 0., 0.), 1., Color::WHITE);
                    d2.draw_model_wires(model, Vector3::new(0., 0., 0.), 1., Color::BLACK);
                }

                for (shape, &CompColor(color)) in (&shapes, &colors).join() {
                    match shape {
                        CompDirectShape::WireFrame { vertices } => {
                            for points in vertices.windows(2) {
                                d2.draw_line_3d(points[0], points[1], color);
                            }
                        }
                        CompDirectShape::Rect { .. } => {}
                    }
                }
            }

            for (shape, &CompColor(color)) in (&shapes, &colors).join() {
                match shape {
                    CompDirectShape::Rect { pos, dimensions } => {
                        let dim = dimensions;
                        d.draw_rectangle(pos.x, pos.y, dim.x, dim.y, color.fade(0.1));
                        d.draw_rectangle_lines(pos.x, pos.y, dim.x, dim.y, color);
                    }
                    CompDirectShape::WireFrame { .. } => {}
                }
            }

            // TODO(cmc): Poor man's cursor
            {
                use raylib::core::math::Vector2 as RayVector2;
                d.draw_circle_sector(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::SKYBLUE);
                d.draw_circle_sector_lines(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::BLUE);
            }

            // TODO(cmc): Poor man's imGUI
            {
                d.draw_rectangle(10, 10, 220, 70, Color::SKYBLUE);
                d.draw_rectangle_lines(10, 10, 220, 70, Color::BLUE);
                d.draw_text("Camera default controls:", 20, 20, 10, Color::BLACK);
                d.draw_text("- Move with keys: W, A, S, D", 40, 40, 10, Color::DARKGRAY);
                d.draw_text("- Mouse wheel to zoom", 40, 60, 10, Color::DARKGRAY);

                d.draw_fps(swidth, sheight);
            }
        });
    }
}

// -----------------------------------------------------------------------------

mod hacks {
    use crate::maths::prelude::*;
    use cgmath::Matrix4 as CGMat4;
    use raylib::{core::math::Matrix as RMat4, ffi::Matrix as c_matrix};

    extern "C" {
        fn GetMatrixProjection() -> c_matrix;
        fn GetMatrixModelview() -> c_matrix;
    }

    // TODO(cmc): def shouldnt be here
    #[rustfmt::skip]
    fn to_mat4(m: RMat4) -> Mat4 {
        let m = m.to_array();
        CGMat4::new(
            m[0],  m[1],  m[2],  m[3],
            m[4],  m[5],  m[6],  m[7],
            m[8],  m[9],  m[10], m[11],
            m[12], m[13], m[14], m[15],
        ).into()
    }

    pub fn get_matrix_projection() -> Mat4 {
        let m: RMat4 = unsafe { GetMatrixProjection() }.into();
        to_mat4(m)
    }
    pub fn get_matrix_modelview() -> Mat4 {
        let m: RMat4 = unsafe { GetMatrixModelview() }.into();
        to_mat4(m)
    }
}
