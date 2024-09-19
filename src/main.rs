mod model;
mod nanolottie;
mod shape_rendering;

use {
    geometric_algebra::{
        ppga3d::{Rotor, Translator},
        GeometricProduct, One,
    },
    macroquad::prelude::*,
    shape_rendering::{raw_miniquad, Vertex3f},
};

#[macroquad::main("Lottie Example")]
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

            let projection_matrix = contrast_renderer::utils::matrix_multiplication(
                &contrast_renderer::utils::perspective_projection(
                    std::f32::consts::PI * 0.5,
                    screen_width() / screen_height(),
                    1.0,
                    1000.0,
                ),
                &contrast_renderer::utils::motor3d_to_mat4(
                    &Translator::new(1.5, 0.0, 0.0, -0.5 * 2.0).geometric_product(Rotor::one()),
                ),
            );

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            gl.quad_context.apply_pipeline(&stage.fill_solid_pipeline);

            gl.quad_context
                .begin_default_pass(miniquad::PassAction::Nothing);
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
                .apply_pipeline(&stage.fill_rational_quadratic_curve_pipeline);

            gl.quad_context
                .begin_default_pass(miniquad::PassAction::Nothing);
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

            let begin_offset = stage.shape2.vertex_offsets[0];
            let end_offset = stage.shape2.vertex_offsets[1];
            let vertex_size = std::mem::size_of::<Vertex3f>();
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
