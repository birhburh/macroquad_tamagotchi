mod model;
mod nanolottie;
mod path_rendering;

use {
    glam::{Affine2, Mat2},
    macroquad::{
        miniquad::{
            conf::{AppleGfxApi, Platform},
            BufferSource, PassAction, TextureFormat, TextureParams, UniformsSource,
        },
        prelude::*,
    },
    path_rendering::{shader, Builder, PathCmd, Rasterizer, Stage, ATLAS_SIZE},
    tiny_skia_path::{PathSegment, Point},
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

fn render_node(node: &usvg::Node, builder: &mut Builder, global_transform: Affine2) {
    match node {
        usvg::Node::Path(ref p) => {
            let t = node.abs_transform();
            let mut transform = global_transform;

            transform *= Affine2::from_mat2_translation(
                Mat2::from_cols_array(&[t.sx as f32, t.ky as f32, t.kx as f32, t.sy as f32]),
                Vec2::new(t.tx as f32, t.ty as f32),
            );

            let mut path = Vec::new();
            for segment in p.data().segments() {
                match segment {
                    PathSegment::MoveTo(Point { x, y }) => {
                        path.push(PathCmd::Move(Vec2::new(x as f32, y as f32)));
                    }
                    PathSegment::LineTo(Point { x, y }) => {
                        path.push(PathCmd::Line(Vec2::new(x as f32, y as f32)));
                    }
                    PathSegment::CubicTo(
                        Point { x: x1, y: y1 },
                        Point { x: x2, y: y2 },
                        Point { x, y },
                    ) => {
                        path.push(PathCmd::Cubic(
                            Vec2::new(x1 as f32, y1 as f32),
                            Vec2::new(x2 as f32, y2 as f32),
                            Vec2::new(x as f32, y as f32),
                        ));
                    }
                    PathSegment::QuadTo(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }) => {
                        path.push(PathCmd::Quadratic(
                            Vec2::new(x1 as f32, y1 as f32),
                            Vec2::new(x2 as f32, y2 as f32),
                        ));
                    }
                    PathSegment::Close => {
                        path.push(PathCmd::Close);
                    }
                }
            }

            if let Some(ref f) = p.fill() {
                if let usvg::Paint::Color(color) = f.paint() {
                    builder.color = [color.red, color.green, color.blue, f.opacity().to_u8()];
                    let mut rasterizer = Rasterizer::new();
                    rasterizer.fill(&path, transform);
                    rasterizer.finish(builder);
                }
            }

            if let Some(ref s) = p.stroke() {
                if let usvg::Paint::Color(color) = s.paint() {
                    builder.color = [color.red, color.green, color.blue, s.opacity().to_u8()];
                    let mut rasterizer = Rasterizer::new();
                    rasterizer.stroke(&path, s.width().get() as f32, transform);
                    rasterizer.finish(builder);
                }
            }
        }
        usvg::Node::Group(ref g) => {
            render_nodes(g, builder, global_transform);
        }
        _ => {}
    }
}

fn render_nodes(group: &usvg::Group, builder: &mut Builder, global_transform: Affine2) {
    for child in group.children() {
        render_node(&child, builder, global_transform);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // let model = nanolottie::load_lottie_file(false);
    // dbg!(&model);

    let svg_data = include_bytes!("../res/Ghostscript_Tiger.svg");
    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default()).unwrap();

    let mut stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        Stage::new(ctx)
    };

    let mut resize = true;
    let mut saved_width = 0.0;
    let mut saved_height = 0.0;

    loop {
        clear_background(DARKGRAY);

        if resize && (screen_width() != saved_width || screen_height() != saved_height) {
            let gl = unsafe { get_internal_gl() };

            let tiger = false;

            saved_width = screen_width();
            saved_height = screen_height();

            stage.builder = Builder::new();
            let side_size = if tiger {
                tree.size().width().min(tree.size().height())
            } else {
                50.0
            };
            let mut scale = if screen_width() < screen_height() {
                screen_width() / side_size * 0.9
            } else {
                screen_height() / side_size * 0.9
            };

            let mut transform = Affine2::from_translation(Vec2::from_array([
                screen_width() / 2.0 - side_size * scale / 2.0,
                screen_height() / 2.0 - side_size * scale / 2.0,
            ]));
            transform *= Affine2::from_scale([scale, scale].into());

            // resize = false;
            // scale = 2.0;
            // transform = Affine2::from_scale([scale, scale].into());

            if tiger {
                render_nodes(&tree.root(), &mut stage.builder, transform);
            } else {
                let mut rasterizer = Rasterizer::new();
                stage.builder.color = [15, 201, 52, 255];

                // Let's say it's circle
                rasterizer.fill(
                    &[
                        PathCmd::Move(Vec2::new(0.0, side_size / 2.0)),
                        PathCmd::Quadratic(Vec2::new(0.0, 0.0), Vec2::new(side_size / 2.0, 0.0)),
                        PathCmd::Quadratic(
                            Vec2::new(side_size, 0.0),
                            Vec2::new(side_size, side_size / 2.0),
                        ),
                        PathCmd::Quadratic(
                            Vec2::new(side_size, side_size),
                            Vec2::new(side_size / 2.0, side_size),
                        ),
                        PathCmd::Line(Vec2::new(0.0, side_size / 2.0)),
                        PathCmd::Close,
                    ],
                    transform,
                );
                rasterizer.finish(&mut stage.builder);
            }

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
