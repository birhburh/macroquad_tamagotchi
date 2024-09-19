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

use vertex::Vertex3f;

pub mod raw_miniquad {
    use super::path::Path;
    use super::renderer::Shape;
    use miniquad::*;

    pub struct Stage {
        pub fill_solid_pipeline: Pipeline,
        pub fill_solid_bindings: Bindings,
        pub fill_rational_quadratic_curve_pipeline: Pipeline,
        pub fill_rational_quadratic_curve_bindings: Bindings,
        pub shape2: Shape,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
            let shape2 = Shape::from_paths(&vec![Path::from_circle([0.0, 0.0], 0.5)]).unwrap();

            let vertices = &shape2.vertex_buffer[0..shape2.vertex_offsets[0] as usize];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );

            let indices = &shape2.index_buffer[0..shape2.index_offsets[0] as usize];
            let ptr = indices.as_ptr() as *const u16;
            let len = std::mem::size_of_val(indices) / std::mem::size_of::<u16>();
            let indices = unsafe { std::slice::from_raw_parts(ptr, len) };

            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(indices),
            );

            let fill_solid_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![],
            };

            let fill_solid_shader = ctx
                .new_shader(
                    match ctx.info().backend {
                        Backend::OpenGl => ShaderSource::Glsl {
                            vertex: shader::FILL_VERTEX,
                            fragment: shader::FILL_FRAGMENT,
                        },
                        Backend::Metal => ShaderSource::Msl {
                            program: shader::FILL_METAL,
                        },
                    },
                    shader::meta(),
                )
                .unwrap();

            let fill_solid_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("position", VertexFormat::Float2)],
                fill_solid_shader,
                PipelineParams {
                    primitive_type: PrimitiveType::TriangleStrip,
                    ..Default::default()
                },
            );

            let begin_offset: usize = shape2.vertex_offsets[0];
            let end_offset = shape2.vertex_offsets[1];
            let vertices = &shape2.vertex_buffer[begin_offset..end_offset];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(vertices),
            );

            let vertex_size = std::mem::size_of::<super::Vertex3f>();
            let indices: Vec<u16> =
                (0..((end_offset - begin_offset) / vertex_size) as u16).collect();

            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );

            let fill_rational_quadratic_curve_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![],
            };
            let fill_rational_quadratic_curve_shader = ctx
                .new_shader(
                    match ctx.info().backend {
                        Backend::OpenGl => ShaderSource::Glsl {
                            vertex: shader::QUADRATIC_VERTEX,
                            fragment: shader::QUADRATIC_FRAGMENT,
                        },
                        Backend::Metal => ShaderSource::Msl {
                            program: shader::QUADRATIC_METAL,
                        },
                    },
                    shader::meta(),
                )
                .unwrap();
            let fill_rational_quadratic_curve_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("position", VertexFormat::Float2),
                    VertexAttribute::new("in_weights", VertexFormat::Float3),
                ],
                fill_rational_quadratic_curve_shader,
                PipelineParams {
                    primitive_type: PrimitiveType::Triangles,
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
            );

            Stage {
                fill_solid_pipeline,
                fill_solid_bindings,
                fill_rational_quadratic_curve_pipeline,
                fill_rational_quadratic_curve_bindings,
                shape2,
            }
        }
    }

    pub mod shader {
        use miniquad::*;

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
void main() {
    gl_FragColor = vec4(0.1, 0.5, 0.2, 1.0);
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

vec4 coverage(bool keep) {
    return keep ? vec4(0.1, 0.5, 0.2, 1.0) : vec4(0.0);
}

void main() {
    gl_FragColor = coverage((weights.x * weights.x - weights.y * weights.z) <= 0.0);
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

        pub fn meta() -> ShaderMeta {
            ShaderMeta {
                images: vec![],
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("transform_row_0", UniformType::Float4),
                        UniformDesc::new("transform_row_1", UniformType::Float4),
                        UniformDesc::new("transform_row_2", UniformType::Float4),
                        UniformDesc::new("transform_row_3", UniformType::Float4),
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
        }
    }
}
