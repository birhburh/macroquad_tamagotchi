mod model;
mod nanolottie;
mod path_rendering;

use {
    macroquad::prelude::*,
    miniquad::PassAction,
    path_rendering::{shader, Stage, ATLAS_SIZE},
};

fn window_conf() -> Conf {
    let sample_count = 1;
    let high_dpi = true;
    Conf {
        window_title: format!(
            "Lottie Example (sample_count = {sample_count}, high_dpi = {high_dpi})"
        )
        .to_owned(),
        platform: miniquad::conf::Platform {
            apple_gfx_api: miniquad::conf::AppleGfxApi::Metal,
            blocking_event_loop: true,
            ..Default::default()
        },
        high_dpi,
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

        Stage::new(ctx)
    };

    loop {
        clear_background(DARKGRAY);

        // draw_lottie(&model);

        {
            let mut gl = unsafe { get_internal_gl() };

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            gl.quad_context
                .begin_default_pass(PassAction::clear_color(1.0, 1.0, 1.0, 1.0));

            gl.quad_context.apply_pipeline(&stage.pipeline);
            gl.quad_context.apply_bindings(&stage.bindings);

            gl.quad_context
                .apply_uniforms(miniquad::UniformsSource::table(&shader::Uniforms {
                    res: [screen_width(), screen_height()],
                    atlas_size: [ATLAS_SIZE as f32, ATLAS_SIZE as f32],
                }));

            gl.quad_context
                .draw(0, stage.builder.indices.len() as i32, 1);

            gl.quad_context.end_render_pass();
        }

        next_frame().await;
    }
}
