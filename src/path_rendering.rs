// Most of it copied from https://github.com/Lichtso/contrast_renderer
mod curve;
mod error;
mod fill;
mod path;
mod renderer;
mod safe_float;
mod stroke;
pub mod utils;
pub mod vertex;
pub extern crate ttf_parser;
mod text;

const OPEN_SANS_TTF: &[u8] = include_bytes!("../fonts/OpenSans-Regular.ttf");

pub mod raw_miniquad {
    use super::path::{LineSegment, Path};
    use super::renderer::Shape;
    use super::text::{paths_of_text, Alignment, Layout, Orientation};
    use super::vertex::{Vertex0, Vertex2f, Vertex3f};
    use super::OPEN_SANS_TTF;
    use macroquad::miniquad::*;

    pub struct Stage {
        pub fill_solid_pipeline: Pipeline,
        pub fill_solid_bindings: Bindings,
        pub fill_integral_quadratic_curve_pipeline: Pipeline,
        pub fill_integral_quadratic_curve_bindings: Bindings,
        pub fill_rational_quadratic_curve_pipeline: Pipeline,
        pub fill_rational_quadratic_curve_bindings: Bindings,
        pub color_cover_pipeline: Pipeline,
        pub color_cover_bindings: Bindings,
        pub shape2: Shape,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
            let font_face = ttf_parser::Face::from_slice(OPEN_SANS_TTF, 0).unwrap();
            let mut paths = paths_of_text(
                &font_face,
                &Layout {
                    size: 2.7.into(),
                    orientation: Orientation::LeftToRight,
                    major_alignment: Alignment::Center,
                    minor_alignment: Alignment::Center,
                },
                "W",
                // "H",
                // "O",
                // "WHO",
                // "Hego",
                // "H",
                // "HW",
                // "Hello World",
            );
            for path in &mut paths {
                path.reverse();
            }
            paths[0].stroke_options = None;
            // let shape2 = Shape::from_paths(&paths).unwrap();

            // let shape2 = Shape::from_paths(&vec![Path::from_circle([0.0, 0.0], 0.5)]).unwrap();

            // let mut path = Path::from_circle([0.0, 0.0], 1.0);
            let mut path = Path::default();
            path.start = [0., 0.].into();
            path.push_line(LineSegment {
                control_points: [[1., 1.].into()],
            });
            path.push_line(LineSegment {
                control_points: [[0., 1.].into()],
            });
            let mut path2 = Path::default();
            path2.start = [0., 0.].into();
            path2.push_line(LineSegment {
                control_points: [[1., 1.].into()],
            });
            path2.push_line(LineSegment {
                control_points: [[0.5, 1.].into()],
            });
            let paths = vec![path, path2, Path::from_circle([0.0, 0.0], 0.5)];
            let shape2 = Shape::from_paths(&paths).unwrap();

            let mut fill_solid_pipeline = None;
            let mut fill_solid_bindings = None;
            let mut fill_integral_quadratic_curve_pipeline = None;
            let mut fill_integral_quadratic_curve_bindings = None;
            let mut fill_rational_quadratic_curve_pipeline = None;
            let mut fill_rational_quadratic_curve_bindings = None;
            for (
                begin_offset,
                end_offset,
                vertex_size,
                vertex_shader,
                fragment_shader,
                metal_shader,
                attributes,
                pipeline,
                bindings,
            ) in [
                (
                    0,
                    shape2.vertex_offsets[0],
                    std::mem::size_of::<Vertex0>(),
                    shader::FILL_VERTEX,
                    shader::FILL_FRAGMENT,
                    shader::FILL_METAL,
                    vec![VertexAttribute::new("position", VertexFormat::Float2)],
                    &mut fill_solid_pipeline,
                    &mut fill_solid_bindings,
                ),
                (
                    shape2.vertex_offsets[0],
                    shape2.vertex_offsets[1],
                    std::mem::size_of::<Vertex2f>(),
                    shader::INTEGRAL_QUADRATIC_VERTEX,
                    shader::INTEGRAL_QUADRATIC_FRAGMENT,
                    shader::INTEGRAL_QUADRATIC_METAL,
                    vec![
                        VertexAttribute::new("position", VertexFormat::Float2),
                        VertexAttribute::new("in_weights", VertexFormat::Float2),
                    ],
                    &mut fill_integral_quadratic_curve_pipeline,
                    &mut fill_integral_quadratic_curve_bindings,
                ),
                (
                    shape2.vertex_offsets[2],
                    shape2.vertex_offsets[3],
                    std::mem::size_of::<Vertex3f>(),
                    shader::QUADRATIC_VERTEX,
                    shader::QUADRATIC_FRAGMENT,
                    shader::QUADRATIC_METAL,
                    vec![
                        VertexAttribute::new("position", VertexFormat::Float2),
                        VertexAttribute::new("in_weights", VertexFormat::Float3),
                    ],
                    &mut fill_rational_quadratic_curve_pipeline,
                    &mut fill_rational_quadratic_curve_bindings,
                ),
            ] {
                let vertices = &shape2.vertex_buffer[begin_offset..end_offset];
                let vertex_buffer = ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&vertices),
                );
                let indices: Vec<u16> =
                    (0..((end_offset - begin_offset) / vertex_size) as u16).collect();

                let index_buffer = ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&indices),
                );

                *bindings = Some(Bindings {
                    vertex_buffers: vec![vertex_buffer],
                    index_buffer,
                    images: vec![],
                });

                let shader = ctx
                    .new_shader(
                        match ctx.info().backend {
                            Backend::OpenGl => ShaderSource::Glsl {
                                vertex: vertex_shader,
                                fragment: fragment_shader,
                            },
                            Backend::Metal => ShaderSource::Msl {
                                program: metal_shader,
                            },
                        },
                        shader::meta(),
                    )
                    .unwrap();

                *pipeline = Some(ctx.new_pipeline(
                    &[BufferLayout::default()],
                    &attributes,
                    shader,
                    PipelineParams {
                        primitive_type: PrimitiveType::Triangles,
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::One,
                            BlendFactor::One,
                        )),
                        ..Default::default()
                    },
                ));
            }

            let begin_offset: usize = shape2.vertex_offsets[5];
            let end_offset = shape2.vertex_offsets[6];
            let vertices = &shape2.vertex_buffer[begin_offset..end_offset];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(vertices),
            );

            let vertex_size = std::mem::size_of::<Vertex0>();
            let indices: Vec<u16> =
                (0..((end_offset - begin_offset) / vertex_size) as u16).collect();

            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );

            let color_img = ctx.new_render_texture(TextureParams {
                width: 0,
                height: 0,
                format: TextureFormat::RGBA8,
                ..Default::default()
            });
            let color_cover_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![color_img],
            };
            let color_cover_shader = ctx
                .new_shader(
                    match ctx.info().backend {
                        Backend::OpenGl => ShaderSource::Glsl {
                            vertex: shader::COVER_VERTEX,
                            fragment: shader::COVER_FRAGMENT,
                        },
                        Backend::Metal => ShaderSource::Msl {
                            program: shader::COVER_METAL,
                        },
                    },
                    shader::cover_meta(),
                )
                .unwrap();
            let color_cover_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("position", VertexFormat::Float2)],
                color_cover_shader,
                PipelineParams {
                    primitive_type: PrimitiveType::Triangles,
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Zero,
                        BlendFactor::Value(BlendValue::SourceColor),
                    )),
                    ..Default::default()
                },
            );

            let fill_solid_pipeline = fill_solid_pipeline.unwrap();
            let fill_solid_bindings = fill_solid_bindings.unwrap();
            let fill_integral_quadratic_curve_pipeline =
                fill_integral_quadratic_curve_pipeline.unwrap();
            let fill_integral_quadratic_curve_bindings =
                fill_integral_quadratic_curve_bindings.unwrap();
            let fill_rational_quadratic_curve_pipeline =
                fill_rational_quadratic_curve_pipeline.unwrap();
            let fill_rational_quadratic_curve_bindings =
                fill_rational_quadratic_curve_bindings.unwrap();
            Stage {
                fill_solid_pipeline,
                fill_solid_bindings,
                fill_integral_quadratic_curve_pipeline,
                fill_integral_quadratic_curve_bindings,
                fill_rational_quadratic_curve_pipeline,
                fill_rational_quadratic_curve_bindings,
                color_cover_pipeline,
                color_cover_bindings,
                shape2,
            }
        }
    }

    pub mod shader {
        use macroquad::miniquad::*;

        pub const FILL_VERTEX: &str = r#"#version 100
precision lowp float;

uniform vec4 transform_row_0;
uniform vec4 transform_row_1;
uniform vec4 transform_row_2;
uniform vec4 transform_row_3;

attribute vec2 position;

void main() {
    mat4 instance = mat4(transform_row_0, transform_row_1,
                         transform_row_2, transform_row_3);
    gl_Position = instance * vec4(position, 0.0, 1.0);
}
"#;

        pub const FILL_FRAGMENT: &str = r#"#version 100
precision lowp float;
uniform vec4 in_color;

void main() {
    gl_FragColor = in_color * (gl_FrontFacing ? 1.0 / 255.0 : 16.0 / 255.0);
}"#;

        pub const FILL_METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4 transform_row_0;
        float4 transform_row_1;
        float4 transform_row_2;
        float4 transform_row_3;
    };

    struct Vertex
    {
        float2 position      [[attribute(0)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        float4x4 instance = float4x4(uniforms.transform_row_0,
                                     uniforms.transform_row_1,
                                     uniforms.transform_row_2,
                                     uniforms.transform_row_3);
        out.position = instance * float4(v.position, 0.0, 1.0);

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return float4(0.1, 0.5, 0.2, 1.0);
    }
"#;

        pub const QUADRATIC_VERTEX: &str = r#"#version 100
precision lowp float;

uniform vec4 transform_row_0;
uniform vec4 transform_row_1;
uniform vec4 transform_row_2;
uniform vec4 transform_row_3;

attribute vec2 position;
attribute vec3 in_weights;

varying vec3 weights;

void main() {
    mat4 instance = mat4(transform_row_0, transform_row_1,
                         transform_row_2, transform_row_3);
    gl_Position = instance * vec4(position, 0.0, 1.0);
    weights = in_weights;
}
"#;

        pub const QUADRATIC_FRAGMENT: &str = r#"#version 100
precision lowp float;

varying vec3 weights;
uniform vec4 in_color;

void main() {
    if ((weights.x * weights.x - weights.y * weights.z) <= 0.0)
        gl_FragColor = in_color * (gl_FrontFacing ? 16.0 / 255.0 : 1.0 / 255.0);
    else
        discard;
}"#;

        pub const QUADRATIC_METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4 transform_row_0;
        float4 transform_row_1;
        float4 transform_row_2;
        float4 transform_row_3;
        float4 in_color;
    };

    struct Vertex
    {
        float2 position      [[attribute(0)]];
        float3 in_weights    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float3 weights [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        float4x4 instance = float4x4(uniforms.transform_row_0,
                                     uniforms.transform_row_1,
                                     uniforms.transform_row_2,
                                     uniforms.transform_row_3);
        out.position = instance * float4(v.position, 0.0, 1.0);
        out.weights = v.in_weights;

        return out;
    }

    float4 coverage(bool keep) {
        return keep ? float4(0.1, 0.5, 0.2, 1.0) : float4(0.0);
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return coverage((in.weights.x * in.weights.x - in.weights.y * in.weights.z) <= 0.0);
    }
"#;

        pub const INTEGRAL_QUADRATIC_VERTEX: &str = r#"#version 100
precision lowp float;

uniform vec4 transform_row_0;
uniform vec4 transform_row_1;
uniform vec4 transform_row_2;
uniform vec4 transform_row_3;

attribute vec2 position;
attribute vec2 in_weights;

varying vec2 weights;

void main() {
    mat4 instance = mat4(transform_row_0, transform_row_1,
                         transform_row_2, transform_row_3);
    gl_Position = instance * vec4(position, 0.0, 1.0);
    weights = in_weights;
}
"#;

        pub const INTEGRAL_QUADRATIC_FRAGMENT: &str = r#"#version 100
precision lowp float;
varying vec2 weights;

uniform vec4 in_color;

void main() {
    if ((weights.x * weights.x - weights.y) <= 0.0)
        gl_FragColor = in_color * (gl_FrontFacing ? 16.0 / 255.0 : 1.0 / 255.0);
    else
        discard;
}"#;

        pub const INTEGRAL_QUADRATIC_METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4 transform_row_0;
        float4 transform_row_1;
        float4 transform_row_2;
        float4 transform_row_3;
        float4 in_color;
    };

    struct Vertex
    {
        float2 position      [[attribute(0)]];
        float2 in_weights    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 weights [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        float4x4 instance = float4x4(uniforms.transform_row_0,
                                     uniforms.transform_row_1,
                                     uniforms.transform_row_2,
                                     uniforms.transform_row_3);
        out.position = instance * float4(v.position, 0.0, 1.0);
        out.weights = v.in_weights;

        return out;
    }

    float4 coverage(bool keep) {
        return keep ? float4(0.1, 0.5, 0.2, 1.0) : float4(0.0);
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return coverage((in.weights.x * in.weights.x - in.weights.y) <= 0.0);
    }
"#;
        pub fn meta() -> ShaderMeta {
            ShaderMeta {
                images: vec![],
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("transform_row_0", UniformType::Float4),
                        UniformDesc::new("transform_row_1", UniformType::Float4),
                        UniformDesc::new("transform_row_2", UniformType::Float4),
                        UniformDesc::new("transform_row_3", UniformType::Float4),
                        UniformDesc::new("in_color", UniformType::Float4),
                    ],
                },
            }
        }

        #[repr(C)]
        pub struct Uniforms {
            pub transform_row_0: [f32; 4],
            pub transform_row_1: [f32; 4],
            pub transform_row_2: [f32; 4],
            pub transform_row_3: [f32; 4],
            pub in_color: [f32; 4],
        }

        pub const COVER_VERTEX: &str = r#"#version 100
            precision lowp float;

            uniform vec4 in_rect;

            attribute vec2 position;

            varying vec2 _coord2;

            void main() {
                _coord2 = mix(in_rect.xy, in_rect.zw, position * 0.5 + 0.5);
	            gl_Position = vec4(_coord2 * 2.0 - 1.0, 0.0, 1.0);
            }
        "#;

        pub const COVER_FRAGMENT: &str = r#"#version 100
            precision lowp float;

            uniform sampler2D tex;
            uniform vec4 in_color;

            varying vec2 _coord2;

            void main() {
                // Get samples for -2/3 and -1/3
                vec2 valueL = texture2D(tex, vec2(_coord2.x, _coord2.y)).yz * 255.0;
                vec2 lowerL = mod(valueL, 16.0);
                vec2 upperL = (valueL - lowerL) / 16.0;
                vec2 alphaL = min(abs(upperL - lowerL), 2.0);

                // Get samples for 0, +1/3, and +2/3
                vec3 valueR = texture2D(tex, _coord2).xyz * 255.0;
                vec3 lowerR = mod(valueR, 16.0);
                vec3 upperR = (valueR - lowerR) / 16.0;
                vec3 alphaR = min(abs(upperR - lowerR), 2.0);

                // Average the energy over the pixels on either side
                vec4 rgba = vec4(
                    (alphaR.x + alphaR.y + alphaR.z) / 6.0,
                    (alphaL.y + alphaR.x + alphaR.y) / 6.0,
                    (alphaL.x + alphaL.y + alphaR.x) / 6.0,
                    0.0);

                // Optionally scale by a color
                gl_FragColor = in_color.a == 0.0 ? 1.0 - rgba : in_color * rgba;
                // gl_FragColor = vec4(0.0);
            }
        "#;

        pub const COVER_METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4 transform_row_0;
        float4 transform_row_1;
        float4 transform_row_2;
        float4 transform_row_3;
        float4 in_color;
    };

    struct Vertex
    {
        float2 position      [[attribute(0)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        float4x4 instance = float4x4(uniforms.transform_row_0,
                                     uniforms.transform_row_1,
                                     uniforms.transform_row_2,
                                     uniforms.transform_row_3);
        out.position = instance * float4(v.position, 0.0, 1.0);
        out.color = uniforms.in_color;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]])
    {
        return float4(in.color.rgb * in.color.a, in.color.a);
    }
"#;

        pub fn cover_meta() -> ShaderMeta {
            ShaderMeta {
                images: vec!["tex".to_string()],
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("transform_row_0", UniformType::Float4),
                        UniformDesc::new("transform_row_1", UniformType::Float4),
                        UniformDesc::new("transform_row_2", UniformType::Float4),
                        UniformDesc::new("transform_row_3", UniformType::Float4),
                        UniformDesc::new("in_color", UniformType::Float4),
                        UniformDesc::new("in_rect", UniformType::Float4),
                    ],
                },
            }
        }

        #[repr(C)]
        pub struct CoverUniforms {
            pub transform_row_0: [f32; 4],
            pub transform_row_1: [f32; 4],
            pub transform_row_2: [f32; 4],
            pub transform_row_3: [f32; 4],
            pub in_color: [f32; 4],
            pub in_rect: [f32; 4],
        }
    }
}
