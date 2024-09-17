mod model;
use {
    lottie::prelude::Bezier,
    macroquad::prelude::*,
    model::{LayerContent, Model, Shape},
    nanoserde::{DeJson, SerJson},
    serde_json,
    std::fs,
};

fn load_lottie_file(compare_with_serde: bool) -> Model {
    let path = "assets/glaxnimate_white_triangle.json";
    // let path = "assets/glaxnimate_triangle.json";
    // let path = "assets/glaxnimate_rectangles.json";
    // let path = "pylottie_circle.json";
    // let path = "../lottie-rs/fixtures/ui/bouncy_ball.json";
    // let path = "../lottie-rs/fixtures/ui/7148-the-nyan-cat.json";
    // let path = "../lottie-rs/fixtures/ui/delete.json";

    let data = fs::read_to_string(path).expect("Unable to read file");

    let s_model: Option<lottie::Model> = if compare_with_serde {
        Some(serde_json::from_str(&data).expect("serde cannot deserialize model"))
    } else {
        None
    };
    // dbg!(&s_model);

    let ns_model: Model =
        DeJson::deserialize_json(&data).expect("nanoserde cannot deserialize model");
    // dbg!(&ns_model);

    if compare_with_serde {
        let s_ser_model = serde_json::to_string(&s_model).expect("serde cannot serialize");
        println!("serde ser: {}", s_ser_model);
        println!("");
        let ns_ser_model = SerJson::serialize_json(&ns_model);
        println!("nanoserde ser: {}", ns_ser_model);
        assert_eq!(s_ser_model, ns_ser_model);
    }
    ns_model
}

fn draw_lottie(model: &Model) {
    for layer in model.layers.iter().rev() {
        match &layer.content {
            LayerContent::Shape(shape) => {
                let mut _fill = None;
                let mut _gradient_fill = None;
                let mut _stroke = None;

                for shape in shape.shapes.iter().rev() {
                    match &shape.shape {
                        Shape::Fill(fill) => _fill = Some(fill.clone()),
                        Shape::GradientFill(fill) => _gradient_fill = Some(fill.clone()),
                        Shape::Stroke(stroke) => _stroke = Some(stroke.clone()),

                        Shape::Rectangle(rectangle) => {
                            if !rectangle.position.animated && !rectangle.size.animated {
                                let x = &rectangle.position.keyframes[0].start_value.0.x;
                                let y = &rectangle.position.keyframes[0].start_value.0.y;
                                let w = &rectangle.size.keyframes[0].start_value.0.x;
                                let h = &rectangle.size.keyframes[0].start_value.0.y;
                                if let Some(fill) = &_fill {
                                    if !fill.color.animated {
                                        let color = fill.color.keyframes[0].start_value;
                                        let color =
                                            Color::from_rgba(color.r, color.g, color.b, 255);
                                        draw_rectangle(*x - *w / 2., *y - *h / 2., *w, *h, color);
                                    }
                                }
                            }
                        }
                        Shape::Path {
                            data, direction, ..
                        } => {
                            if !data.animated {
                                let bezier = &data.keyframes[0].start_value;
                                if let Some(fill) = &_fill {
                                    if !fill.color.animated {
                                        let color = fill.color.keyframes[0].start_value;
                                        let color =
                                            Color::from_rgba(color.r, color.g, color.b, 255);
                                        draw_rectangle(1., 1., 20., 20., color);

                                        // let mut prev_p: Option<Vector2D>;
                                        // match b.verticies.first() {
                                        //     Some(p) => {
                                        //         builder.begin(p.to_point());
                                        //         prev_p = Some(*p);
                                        //     }
                                        //     None => continue,
                                        // }
                                        // for ((p, c1), c2) in b
                                        //     .verticies
                                        //     .iter()
                                        //     .skip(1)
                                        //     .zip(b.out_tangent.iter())
                                        //     .zip(b.in_tangent.iter().skip(1))
                                        // {
                                        //     if let Some(p0) = prev_p {
                                        //         let p1 = p0 + *c1;
                                        //         let p2 = *p + *c2;
                                        //         if c1.approx_eq(&Vector2D::zero()) && c2.approx_eq(&Vector2D::zero()) {
                                        //             builder.line_to(p.to_point());
                                        //         } else if p1.approx_eq(&p2) {
                                        //             builder.quadratic_bezier_to(p1.to_point(), p.to_point());
                                        //         } else {
                                        //             builder.cubic_bezier_to(p1.to_point(), p2.to_point(), p.to_point());
                                        //         }
                                        //     }
                                        //     prev_p = Some(*p);
                                        // }
                                        // if b.closed {
                                        //     let index = b.verticies.len() - 1;
                                        //     builder.cubic_bezier_to(
                                        //         (b.verticies[index] + b.out_tangent[index]).to_point(),
                                        //         (b.verticies[0] + b.in_tangent[0]).to_point(),
                                        //         b.verticies[0].to_point(),
                                        //     );
                                        // }
                                    }
                                }
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[macroquad::main("Lottie Example")]
async fn main() {
    let model = load_lottie_file(false);
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

            let t = get_time();

            gl.quad_context.apply_pipeline(&stage.pipeline);

            gl.quad_context
                .begin_default_pass(miniquad::PassAction::Nothing);
            gl.quad_context.apply_bindings(&stage.bindings);

            for i in 0..10 {
                let t = t + i as f64 * 0.3;

                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &raw_miniquad::shader::Uniforms {
                            offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
                        },
                    ));
                gl.quad_context.draw(0, 6, 1);
            }
            gl.quad_context.end_render_pass();
        }

        next_frame().await;
    }
}

mod raw_miniquad {
    use miniquad::*;

    #[repr(C)]
    struct Vec2 {
        x: f32,
        y: f32,
    }
    #[repr(C)]
    struct Vertex {
        pos: Vec2,
        uv: Vec2,
    }

    pub struct Stage {
        pub pipeline: Pipeline,
        pub bindings: Bindings,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
            #[rustfmt::skip]
            let vertices: [Vertex; 4] = [
                Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
                Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
                Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
                Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
            ];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );

            let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices[..]),
            );

            let pixels: [u8; 4 * 4 * 4] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
                0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ];
            let texture = ctx.new_texture_from_rgba8(4, 4, &pixels);

            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![texture],
            };

            let shader = ctx
                .new_shader(
                    miniquad::ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    shader::meta(),
                )
                .unwrap();

            let pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("pos", VertexFormat::Float2),
                    VertexAttribute::new("uv", VertexFormat::Float2),
                ],
                shader,
                Default::default(),
            );

            Stage { pipeline, bindings }
        }
    }

    pub mod shader {
        use miniquad::*;

        pub const VERTEX: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;

uniform vec2 offset;

varying lowp vec2 texcoord;

void main() {
    gl_Position = vec4(pos + offset, 0, 1);
    texcoord = uv;
}"#;

        pub const FRAGMENT: &str = r#"#version 100
varying lowp vec2 texcoord;

uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, texcoord);
}"#;

        pub fn meta() -> ShaderMeta {
            ShaderMeta {
                images: vec!["tex".to_string()],
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
                },
            }
        }

        #[repr(C)]
        pub struct Uniforms {
            pub offset: (f32, f32),
        }
    }
}
