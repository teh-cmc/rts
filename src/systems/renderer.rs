use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use raylib::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

// TODO(cmc): throw some lightning and shadows in there

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
        ReadStorage<'a, CompVoxelModel>,
        ReadStorage<'a, CompGridPosition>,
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
            voxels,
            grid_pos,
        ) = sys_data;

        let thread = self.0.as_ref().unwrap();
        // TODO(cmc): why on earth does this segfault?
        // let font = rl.read(|rl| rl.get_font_default());
        let (swidth, sheight) = rl.read(|rl| (rl.get_screen_width(), rl.get_screen_height()));
        let (x, y) = rl.read(|rl| (rl.get_mouse_x() as f32, rl.get_mouse_y() as f32));
        // TODO(cmc): only render what's visible in the frustrum.
        rl.draw(&thread, |d| {
            d.clear_background(Color::DARKGRAY);

            let render_start = std::time::Instant::now();
            let mut render_voxels = 0;
            let mut render_triangles = 0;
            {
                let mut d2 = d.begin_mode_3D(cam.raw());

                // Projection and ModelView matrices corresponding to the main
                // camera have now been loaded on the other side of the FFI
                // barrier: grab 'em!
                *m_view.0 = *hacks::get_matrix_modelview();
                *m_proj.0 = *hacks::get_matrix_projection();

                for (e, CompVoxelModel(model), CompGridPosition(world_pos), &CompColor(color)) in
                    (&entities, &voxels, &grid_pos, &colors).join()
                {
                    // TODO(cmc): something smarter
                    let color = if let Some(_) = selected.get(e) {
                        Color::GOLD
                    } else {
                        color
                    };

                    use raylib::core::math::Vector3 as RayVector3;
                    let dims: RayVector3 = (1., 1., 1.).into();

                    let model_stats = model.stats();
                    render_voxels += model_stats.nb_voxels;
                    render_triangles += model_stats.nb_triangles;

                    let voxels: Vec<_> = model
                        .iter()
                        .filter(|(_, voxel)| *voxel)
                        .map(|(pos, _)| **world_pos + *pos)
                        .map(|pos| RayVector3::from((pos.x as f32, pos.y as f32, pos.z as f32)))
                        .collect();
                    for pos in voxels.iter() {
                        d2.draw_cube_v(pos, dims, color);
                    }
                    for pos in voxels.iter() {
                        d2.draw_cube_wires(pos, dims.x, dims.y, dims.z, Color::BLACK);
                    }
                }

                for (shape, &CompColor(color)) in (&shapes, &colors).join() {
                    match shape {
                        CompDirectShape::WireFrame { vertices } => {
                            for points in vertices.windows(2) {
                                d2.draw_line_3d(points[1], points[0], color);
                            }
                        }
                        CompDirectShape::Rect { .. } => {}
                    }
                }

                for _ in (&voxels, &grid_pos).join() {}

                // TODO(cmc): Poor man's axes
                {
                    let p1: Point3 = (0., 0., 0.).into();

                    let x: Point3 = (40., 0., 0.).into();
                    d2.draw_line_3d(p1, x, Color::DARKGREEN);

                    let y: Point3 = (0., 40., 0.).into();
                    d2.draw_line_3d(p1, y, Color::YELLOW);

                    let z: Point3 = (0., 0., 40.).into();
                    d2.draw_line_3d(p1, z, Color::MAROON);
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
            let render_time = render_start.elapsed();

            d.draw_fps(swidth - 100, 10);
            imgui::draw_cursor(d, x, y);
            #[rustfmt::skip]
            imgui::draw_debug_info(
                d,
                10, 10, 0, 0, 10,
                "Camera default controls:".into(),
                &[
                    "- Move with keys: W, A, S, D",
                    "- Mouse wheel to zoom"
                ],
            );
            #[rustfmt::skip]
            imgui::draw_debug_info(
                d,
                10, sheight - 80, 150, 0, 10,
                "Rendering stats:".into(),
                &[
                    format!("- Duration: {:?}", render_time).as_str(),
                    format!("- Voxels: {:#?}", render_voxels).as_str(),
                    format!("- Triangles: {:#?}", render_triangles).as_str(),
                ],
            );
        });
    }
}

mod imgui {
    use super::*;
    use raylib::core::text::measure_text as rl_text_width;

    pub fn draw_cursor(d: &mut RaylibDrawHandle, x: f32, y: f32) {
        use raylib::core::math::Vector2 as RayVector2;
        d.draw_circle_sector(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::SKYBLUE);
        d.draw_circle_sector_lines(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::BLUE);
    }

    pub fn draw_debug_info(
        d: &mut RaylibDrawHandle,
        mut x: i32,
        mut y: i32,
        min_width: i32,
        min_height: i32,
        font_size: i32,
        title: Option<&str>,
        body: &[&str],
    ) {
        const VERTICAL_SPACING: i32 = 10;
        const HORIZONTAL_SPACING: i32 = 10;

        let title_width = title.map(|txt| rl_text_width(txt, font_size));
        let body_width = body.iter().map(|line| rl_text_width(line, font_size)).max();

        let rec_width = Option::max(title_width, body_width);
        if rec_width.is_none() {
            return;
        }

        let height = (title.is_some() as i32 + body.len() as i32)
            * (VERTICAL_SPACING / 2 + font_size)
            + VERTICAL_SPACING;
        let width = rec_width.unwrap() + HORIZONTAL_SPACING;

        let height = i32::max(height, min_height + VERTICAL_SPACING);
        let width = i32::max(width, min_width + HORIZONTAL_SPACING);

        d.draw_rectangle(x, y, width, height, Color::SKYBLUE);
        d.draw_rectangle_lines(x, y, width, height, Color::BLUE);
        x += HORIZONTAL_SPACING / 2;
        y += VERTICAL_SPACING / 2;

        {
            if let Some(title) = title {
                d.draw_text(title, x, y, font_size, Color::BLACK);
                y += font_size + VERTICAL_SPACING / 2;
            }
        }

        {
            for txt in body {
                d.draw_text(txt, x, y, font_size, Color::DARKGRAY);
                y += font_size + VERTICAL_SPACING / 2;
            }
        }
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

// -----------------------------------------------------------------------------

// for (
//     e,
//     &mut CompModel3D(ref mut model),
//     &CompTransform3D(transform),
//     &CompColor(color),
// ) in (&entities, &mut models, &transforms, &colors).join()
// {
//     // TODO(cmc): something smarter
//     let color = if let Some(_) = selected.get(e) {
//         Color::GOLD
//     } else {
//         color
//     };

//     // TODO(cmc): needs interior mutability... or a transform
//     // dedicated system?
//     // model.set_transform(&transform.into());
//     d2.draw_model(
//         model,
//         Vector3::new(transform.w.x, transform.w.y, transform.w.z),
//         1.,
//         color,
//     );

//     // NOTE(cmc): draw_model_wires is bugged to death when
//     // running through emscripten; haven't digged into it yet...
//     // #[cfg(not(target_os = "emscripten"))]
//     // d2.draw_model_wires(model, Vector3::zero(), 1.0, Color::BLACK);
//     #[cfg(target_os = "emscripten")]
//     {
//         d2.draw_cube_wires(
//             Vector3::new(transform.w.x, transform.w.y, transform.w.z),
//             transform.x.x,
//             transform.y.y,
//             transform.z.z,
//             Color::BLACK,
//         );
//     }
// }
