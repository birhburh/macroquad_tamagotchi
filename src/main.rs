mod model;
mod nanolottie;
mod path_rendering;

use {
    geometric_algebra::{
        ppga3d::{Rotor, Translator},
        GeometricProduct, One,
    },
    macroquad::prelude::*,
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

#[macroquad::main(window_conf)]
async fn main() {
    // let model = nanolottie::load_lottie_file(false);
    // dbg!(&model);

    let stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        raw_miniquad::Stage::new(ctx)
    };

    loop {
        clear_background(LIGHTGRAY);

        // draw_lottie(&model);

        {
            let mut gl = unsafe { get_internal_gl() };

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            gl.quad_context
                .begin_default_pass(miniquad::PassAction::Clear {
                    stencil: Some(0),
                    color: Default::default(),
                    depth: Default::default(),
                });

            let projection_matrix = matrix_multiplication(
                &perspective_projection(
                    std::f32::consts::PI * 0.5,
                    screen_width() / screen_height(),
                    1.0,
                    1000.0,
                ),
                &motor3d_to_mat4(
                    &Translator::new(1.5, 0.0, 0.0, -0.5 * 3.0).geometric_product(Rotor::one()),
                ),
            );

            gl.quad_context.apply_pipeline(&stage.fill_solid_pipeline);
            gl.quad_context.apply_bindings(&stage.fill_solid_bindings);

            gl.quad_context
                .apply_uniforms(miniquad::UniformsSource::table(
                    &raw_miniquad::shader::Uniforms {
                        transform_row_0: projection_matrix[0].into(),
                        transform_row_1: projection_matrix[1].into(),
                        transform_row_2: projection_matrix[2].into(),
                        transform_row_3: projection_matrix[3].into(),
                    },
                ));

            gl.quad_context.draw(
                0,
                (stage.shape2.index_offsets[0] / std::mem::size_of::<u16>())
                    .try_into()
                    .unwrap(),
                1,
            );

            gl.quad_context
                .apply_pipeline(&stage.fill_integral_quadratic_curve_pipeline);
            gl.quad_context
                .apply_bindings(&stage.fill_integral_quadratic_curve_bindings);

            gl.quad_context
                .apply_uniforms(miniquad::UniformsSource::table(
                    &raw_miniquad::shader::Uniforms {
                        transform_row_0: projection_matrix[0].into(),
                        transform_row_1: projection_matrix[1].into(),
                        transform_row_2: projection_matrix[2].into(),
                        transform_row_3: projection_matrix[3].into(),
                    },
                ));

            let begin_offset = stage.shape2.vertex_offsets[0];
            let end_offset = stage.shape2.vertex_offsets[1];
            let vertex_size = std::mem::size_of::<Vertex2f>();
            gl.quad_context.draw(
                0,
                ((end_offset - begin_offset) / vertex_size)
                    .try_into()
                    .unwrap(),
                1,
            );

            gl.quad_context
                .apply_pipeline(&stage.fill_rational_quadratic_curve_pipeline);
            gl.quad_context
                .apply_bindings(&stage.fill_rational_quadratic_curve_bindings);

            gl.quad_context
                .apply_uniforms(miniquad::UniformsSource::table(
                    &raw_miniquad::shader::Uniforms {
                        transform_row_0: projection_matrix[0].into(),
                        transform_row_1: projection_matrix[1].into(),
                        transform_row_2: projection_matrix[2].into(),
                        transform_row_3: projection_matrix[3].into(),
                    },
                ));

            let begin_offset = stage.shape2.vertex_offsets[2];
            let end_offset = stage.shape2.vertex_offsets[3];
            let vertex_size = std::mem::size_of::<Vertex3f>();
            gl.quad_context.draw(
                0,
                ((end_offset - begin_offset) / vertex_size)
                    .try_into()
                    .unwrap(),
                1,
            );

            gl.quad_context
                .apply_pipeline(&stage.color_cover_pipeline);
            gl.quad_context
                .apply_bindings(&stage.color_cover_bindings);

            gl.quad_context
                .apply_uniforms(miniquad::UniformsSource::table(
                    &raw_miniquad::shader::UniformsWithColor {
                        transform_row_0: projection_matrix[0].into(),
                        transform_row_1: projection_matrix[1].into(),
                        transform_row_2: projection_matrix[2].into(),
                        transform_row_3: projection_matrix[3].into(),
                        in_color: [0.1, 0.5, 0.2, 1.0],
                    },
                ));

            let begin_offset = stage.shape2.vertex_offsets[4];
            let end_offset = stage.shape2.vertex_offsets[5];
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

        next_frame().await;
    }
}
