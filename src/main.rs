mod model;
mod nanolottie;
mod path_rendering;

use {
    geometric_algebra::{
        ppga3d::{Motor, Rotor, Translator},
        GeometricProduct, One,
    },
    macroquad::prelude::*,
    miniquad::{PassAction, TextureFormat, TextureParams},
    path_rendering::{
        raw_miniquad,
        utils::{matrix_multiplication, motor3d_to_mat4, perspective_projection},
        vertex::{Vertex0, Vertex2f, Vertex3f},
    },
};

fn window_conf() -> Conf {
    let sample_count = 1;
    Conf {
        window_title: format!("Lottie Example (sample_count = {sample_count})").to_owned(),
        platform: miniquad::conf::Platform {
            apple_gfx_api: miniquad::conf::AppleGfxApi::OpenGl,
            ..Default::default()
        },
        // high_dpi: true,
        sample_count,
        ..Default::default()
    }
}

// 6x subpixel AA pattern
//
//   R = (f(x - 2/3, y) + f(x - 1/3, y) + f(x, y)) / 3
//   G = (f(x - 1/3, y) + f(x, y) + f(x + 1/3, y)) / 3
//   B = (f(x, y) + f(x + 1/3, y) + f(x + 2/3, y)) / 3
//
// The shader would require three texture lookups if the texture format
// stored data for offsets -1/3, 0, and +1/3 since the shader also needs
// data for offsets -2/3 and +2/3. To avoid this, the texture format stores
// data for offsets 0, +1/3, and +2/3 instead. That way the shader can get
// data for offsets -2/3 and -1/3 with only one additional texture lookup.
//
const JITTER_PATTERN: [[f32; 2]; 6] = [
    [-1. / 12.0, -5. / 12.0],
    [1. / 12.0, 1. / 12.0],
    [3. / 12.0, -1. / 12.0],
    [5. / 12.0, 5. / 12.0],
    [7. / 12.0, -3. / 12.0],
    [9. / 12.0, 3. / 12.0],
];

#[macroquad::main(window_conf)]
async fn main() {
    // let model = nanolottie::load_lottie_file(false);
    // dbg!(&model);

    let mut stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        raw_miniquad::Stage::new(ctx)
    };

    let mut offscreen_width = 0;
    let mut offscreen_height = 0;
    let mut offscreen_pass = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        let color_img = ctx.new_render_texture(TextureParams {
            width: offscreen_width,
            height: offscreen_height,
            format: TextureFormat::RGBA8,
            ..Default::default()
        });
        stage.color_cover_bindings.images[0] = color_img;
        ctx.new_render_pass(color_img, None)
    };

    loop {
        clear_background(DARKGRAY);

        if offscreen_width != screen_width() as u32 && offscreen_height != screen_height() as u32 {
            offscreen_width = screen_width() as u32;
            offscreen_height = screen_height() as u32;
            dbg!((offscreen_width, offscreen_height));
            offscreen_pass = {
                let InternalGlContext {
                    quad_context: ctx, ..
                } = unsafe { get_internal_gl() };
                let color_img = ctx.new_render_texture(TextureParams {
                    width: offscreen_width,
                    height: offscreen_height,
                    format: TextureFormat::RGBA8,
                    ..Default::default()
                });
                stage.color_cover_bindings.images[0] = color_img;

                let new_offscreen_pass = ctx.new_render_pass(color_img, None);

                ctx.delete_render_pass(offscreen_pass);
                new_offscreen_pass
            };
        }

        // draw_lottie(&model);

        {
            let draw_to_texture = true;
            let mut gl = unsafe { get_internal_gl() };

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            if draw_to_texture {
                gl.quad_context.begin_pass(
                    Some(offscreen_pass),
                    PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
                );
            } else {
                gl.quad_context.begin_default_pass(PassAction::Nothing);
            }
            let mut projection_matrix = matrix_multiplication(
                &perspective_projection(
                    std::f32::consts::PI * 0.5,
                    screen_width() / screen_height(),
                    1.0,
                    1000.0,
                ),
                &motor3d_to_mat4(
                    &Translator::new(1.0, 0.0, 0.0, -1.0).geometric_product(Rotor::one()),
                ),
            );

            for j in 0..JITTER_PATTERN.len() {
                let offset = JITTER_PATTERN[j];
                for (pipeline, bindings, begin_offset, end_offset, vertex_size) in [
                    (
                        &stage.fill_solid_pipeline,
                        &stage.fill_solid_bindings,
                        0,
                        stage.shape2.vertex_offsets[0],
                        std::mem::size_of::<Vertex0>(),
                    ),
                    (
                        &stage.fill_integral_quadratic_curve_pipeline,
                        &stage.fill_integral_quadratic_curve_bindings,
                        stage.shape2.vertex_offsets[0],
                        stage.shape2.vertex_offsets[1],
                        std::mem::size_of::<Vertex2f>(),
                    ),
                    (
                        &stage.fill_rational_quadratic_curve_pipeline,
                        &stage.fill_rational_quadratic_curve_bindings,
                        stage.shape2.vertex_offsets[2],
                        stage.shape2.vertex_offsets[3],
                        std::mem::size_of::<Vertex3f>(),
                    ),
                ] {
                    let scale = 1. / screen_dpi_scale();
                    // dbg!(offset);
                    gl.quad_context.apply_pipeline(pipeline);
                    gl.quad_context.apply_bindings(bindings);
                    let mut in_color = [0.0; 4];

                    if j % 2 == 0 {
                        in_color[0] = if j == 0 { 1.0 } else { 0.0 };
                        in_color[1] = if j == 2 { 1.0 } else { 0.0 };
                        in_color[2] = if j == 4 { 1.0 } else { 0.0 };
                    }
                    // projection_matrix = matrix_multiplication(
                    //     &projection_matrix,
                    //     &motor3d_to_mat4(
                    //         &Translator::new(
                    //             1.0,
                    //             (offset[0] * scale) / screen_width(),
                    //             (offset[1] * scale) / screen_height(),
                    //             0.0,
                    //         )
                    //         .geometric_product(Rotor::one()),
                    //     ),
                    // );
                    gl.quad_context
                        .apply_uniforms(miniquad::UniformsSource::table(
                            &raw_miniquad::shader::Uniforms {
                                transform_row_0: projection_matrix[0].into(),
                                transform_row_1: projection_matrix[1].into(),
                                transform_row_2: projection_matrix[2].into(),
                                transform_row_3: projection_matrix[3].into(),
                                in_color,
                            },
                        ));

                    gl.quad_context.draw(
                        0,
                        ((end_offset - begin_offset) / vertex_size)
                            .try_into()
                            .unwrap(),
                        1,
                    );
                }
            }
            gl.quad_context.end_render_pass();

            if draw_to_texture {
                gl.quad_context.begin_default_pass(PassAction::Nothing);

                gl.quad_context.apply_pipeline(&stage.color_cover_pipeline);
                gl.quad_context.apply_bindings(&stage.color_cover_bindings);

                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &raw_miniquad::shader::CoverUniforms {
                            transform_row_0: projection_matrix[0].into(),
                            transform_row_1: projection_matrix[1].into(),
                            transform_row_2: projection_matrix[2].into(),
                            transform_row_3: projection_matrix[3].into(),
                            in_color: [0.0; 4],
                            in_rect: [
                                stage.shape2.convex_box[0].floor(),
                                1. - stage.shape2.convex_box[3].ceil(),
                                stage.shape2.convex_box[2].ceil(),
                                1. - stage.shape2.convex_box[1].floor(),
                            ],
                        },
                    ));

                let begin_offset = stage.shape2.vertex_offsets[5];
                let end_offset = stage.shape2.vertex_offsets[6];
                let vertex_size = std::mem::size_of::<Vertex0>();
                gl.quad_context.draw(
                    0,
                    ((end_offset - begin_offset) / vertex_size)
                        .try_into()
                        .unwrap(),
                    1,
                );

                gl.quad_context.end_render_pass();
            }
        }

        next_frame().await;
    }
}
