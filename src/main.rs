mod model;
mod nanolottie;
mod path_rendering;

use {
    macroquad::{
        miniquad::{
            conf::{AppleGfxApi, Platform},
            BufferSource, PassAction, TextureFormat, TextureParams, UniformsSource,
        },
        prelude::*,
    },
    path_rendering::{shader, Builder, PathCmd, Rasterizer, Stage, Transform, ATLAS_SIZE},
};

fn window_conf() -> Conf {
    let sample_count = 1;
    let high_dpi = false;
    Conf {
        window_title: format!(
            "Lottie Example (sample_count = {sample_count}, high_dpi = {high_dpi})"
        )
        .to_owned(),
        platform: Platform {
            apple_gfx_api: AppleGfxApi::OpenGl,
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

    let mut stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        Stage::new(ctx)
    };

    let mut saved_width = 0.0;
    let mut saved_height = 0.0;

    loop {
        clear_background(DARKGRAY);

        if screen_width() != saved_width || screen_height() != saved_height {
            let mut gl = unsafe { get_internal_gl() };

            saved_width = screen_width();
            saved_height = screen_height();

            let mut rasterizer = Rasterizer::new();
            stage.builder = Builder::new();
            stage.builder.color = [15, 201, 52, 255];

            let side_size = 50.0;
            let scale = if screen_width() < screen_height() {
                screen_width() / side_size
            } else {
                screen_height() / side_size
            } / 2.0;
            dbg!(scale);
            // Let's say it's circle
            rasterizer.fill(
                &[
                    PathCmd::Move(path_rendering::Vec2::new(0.0, 0.0)),
                    PathCmd::Quadratic(
                        path_rendering::Vec2::new(side_size / 2.0, 0.0),
                        path_rendering::Vec2::new(side_size / 2.0, side_size / 2.0),
                    ),
                    PathCmd::Quadratic(
                        path_rendering::Vec2::new(side_size / 2.0, side_size),
                        path_rendering::Vec2::new(0.0, side_size),
                    ),
                    PathCmd::Quadratic(
                        path_rendering::Vec2::new(-side_size / 2.0, side_size),
                        path_rendering::Vec2::new(-side_size / 2.0, side_size / 2.0),
                    ),
                    PathCmd::Quadratic(
                        path_rendering::Vec2::new(-side_size / 2.0, 0.0),
                        path_rendering::Vec2::new(0.0, 0.0),
                    ),
                    PathCmd::Close,
                ],
                Transform::scale(scale).then(Transform::translate(
                    screen_width() / 2.0 + side_size / 2.0 * scale,
                    side_size / 2.0 * scale,
                )),
            );
            rasterizer.finish(&mut stage.builder);
            gl.quad_context.buffer_update(
                stage.bindings.vertex_buffers[0],
                BufferSource::slice(&stage.builder.vertices),
            );
            gl.quad_context.buffer_update(
                stage.bindings.index_buffer,
                BufferSource::slice(&stage.builder.indices),
            );

            gl.quad_context.delete_texture(stage.bindings.images[0]);

            let tex = gl.quad_context.new_texture_from_data_and_format(
                &stage.builder.atlas,
                TextureParams {
                    width: ATLAS_SIZE as u32,
                    height: ATLAS_SIZE as u32,
                    format: TextureFormat::RGBA8,
                    min_filter: FilterMode::Nearest,
                    mag_filter: FilterMode::Nearest,
                    ..Default::default()
                },
            );

            stage.bindings.images[0] = tex;
        }

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
                .apply_uniforms(UniformsSource::table(&shader::Uniforms {
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
