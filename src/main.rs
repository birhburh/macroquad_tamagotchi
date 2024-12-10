mod model;
mod nanolottie;
mod path_rendering;
mod smaa;

use {
    geometric_algebra::{
        ppga3d::{Rotor, Translator},
        GeometricProduct, One,
    },
    macroquad::prelude::*,
    miniquad::{window::screen_size, MipmapFilterMode, PassAction, TextureFormat, TextureParams},
    path_rendering::{
        utils::{matrix_multiplication, motor3d_to_mat4, perspective_projection},
        vertex::{Vertex0, Vertex2f, Vertex3f},
    },
};

fn window_conf() -> Conf {
    let sample_count = 1;
    // let apple_gfx_api = miniquad::conf::AppleGfxApi::Metal;
    let apple_gfx_api = miniquad::conf::AppleGfxApi::OpenGl;
    let high_dpi = true;
    Conf {
        window_title: format!(
            "SMAA Example (high_dpi = {high_dpi}, apple_gfx_api={apple_gfx_api:?})"
        )
        .to_owned(),
        platform: miniquad::conf::Platform {
            apple_gfx_api,
            ..Default::default()
        },
        high_dpi,
        sample_count,
        // fullscreen: true,
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

        path_rendering::raw_miniquad::Stage::new(ctx)
    };

    let mut smaa_stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        smaa::raw_miniquad::Stage::new(ctx)
    };

    let mut offscreen_width = 0;
    let mut offscreen_height = 0;

    loop {
        clear_background(LIGHTGRAY);

        // draw_lottie(&model);

        {
            let mut gl = unsafe { get_internal_gl() };
            let width = screen_size().0 as u32;
            let height = screen_size().1 as u32;
            let u_rt = [
                1.0 / width as f32,
                1.0 / height as f32,
                width as f32,
                height as f32,
            ];

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            if offscreen_width != width && offscreen_height != height {
                offscreen_width = width;
                offscreen_height = height;
                {
                    let InternalGlContext {
                        quad_context: ctx, ..
                    } = unsafe { get_internal_gl() };
                    ctx.delete_render_pass(smaa_stage.render_offscreen_pass);
                    ctx.delete_render_pass(smaa_stage.edge_detect_offscreen_pass);
                    ctx.delete_render_pass(smaa_stage.blend_weight_offscreen_pass);

                    let render_img = ctx.new_render_texture(TextureParams {
                        width: offscreen_width,
                        height: offscreen_height,
                        format: TextureFormat::RGBA8,
                        min_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        mipmap_filter: MipmapFilterMode::None,
                        ..Default::default()
                    });
                    let depth_stencil_img = ctx.new_render_texture(TextureParams {
                        width: offscreen_width,
                        height: offscreen_height,
                        format: TextureFormat::DepthStencil,
                        min_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        mipmap_filter: MipmapFilterMode::None,
                        ..Default::default()
                    });

                    smaa_stage.render_offscreen_pass =
                        ctx.new_render_pass(render_img, Some(depth_stencil_img));
                    smaa_stage.edge_detect_bindings.images[0] = render_img;
                    smaa_stage.neighborhood_blending_bindings.images[0] = render_img;

                    let edge_detect_img = ctx.new_render_texture(TextureParams {
                        width: offscreen_width,
                        height: offscreen_height,
                        format: TextureFormat::RGBA8,
                        min_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        mipmap_filter: MipmapFilterMode::None,
                        ..Default::default()
                    });

                    smaa_stage.edge_detect_offscreen_pass =
                        ctx.new_render_pass(edge_detect_img, None);
                    smaa_stage.blend_weight_bindings.images[0] = edge_detect_img;

                    let blend_weight_img = ctx.new_render_texture(TextureParams {
                        width: offscreen_width,
                        height: offscreen_height,
                        format: TextureFormat::RGBA8,
                        min_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        mipmap_filter: MipmapFilterMode::None,
                        ..Default::default()
                    });
                    smaa_stage.blend_weight_offscreen_pass =
                        ctx.new_render_pass(blend_weight_img, None);
                    smaa_stage.neighborhood_blending_bindings.images[1] = blend_weight_img;
                };
            }

            {
                gl.quad_context.begin_pass(
                    Some(smaa_stage.render_offscreen_pass),
                    miniquad::PassAction::Clear {
                        stencil: Some(0),
                        color: Some((0., 0., 0., 0.)),
                        depth: Default::default(),
                    },
                );

                // gl.quad_context
                //     .begin_default_pass(miniquad::PassAction::Clear {
                //         stencil: Some(0),
                //         color: Default::default(),
                //         depth: Default::default(),
                //     });

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
                        &path_rendering::raw_miniquad::shader::Uniforms {
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
                        &path_rendering::raw_miniquad::shader::Uniforms {
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
                        &path_rendering::raw_miniquad::shader::Uniforms {
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

                gl.quad_context.apply_pipeline(&stage.color_cover_pipeline);
                gl.quad_context.apply_bindings(&stage.color_cover_bindings);

                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &path_rendering::raw_miniquad::shader::UniformsWithColor {
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

            {
                // gl.quad_context
                //     .begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 0.0));

                gl.quad_context.begin_pass(
                    Some(smaa_stage.edge_detect_offscreen_pass),
                    PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
                );

                gl.quad_context
                    .apply_pipeline(&smaa_stage.edge_detect_pipeline);
                gl.quad_context
                    .apply_bindings(&smaa_stage.edge_detect_bindings);
                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &smaa::raw_miniquad::shader::Uniforms { u_rt },
                    ));
                gl.quad_context.draw(0, 3, 1);
                gl.quad_context.end_render_pass();
            }

            {
                // gl.quad_context
                //     .begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 0.0));

                gl.quad_context.begin_pass(
                    Some(smaa_stage.blend_weight_offscreen_pass),
                    PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
                );

                gl.quad_context
                    .apply_pipeline(&smaa_stage.blend_weight_pipeline);
                gl.quad_context
                    .apply_bindings(&smaa_stage.blend_weight_bindings);
                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &smaa::raw_miniquad::shader::Uniforms { u_rt },
                    ));
                gl.quad_context.draw(0, 3, 1);
                gl.quad_context.end_render_pass();
            }

            {
                gl.quad_context.begin_default_pass(PassAction::Nothing);

                gl.quad_context
                    .apply_pipeline(&smaa_stage.neighborhood_blending_pipeline);
                gl.quad_context
                    .apply_bindings(&smaa_stage.neighborhood_blending_bindings);

                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &smaa::raw_miniquad::shader::Uniforms { u_rt },
                    ));

                gl.quad_context.draw(0, 3, 1);

                gl.quad_context.end_render_pass();
            }
        }

        next_frame().await;
    }
}
