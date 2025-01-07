// Most of it copied from https://github.com/micahrj/ochre

mod geom;
mod path;
mod rasterizer;
mod tile_builder_impl;

use {
    geom::{Transform, Vec2},
    macroquad::miniquad::*,
    path::PathCmd,
    rasterizer::Rasterizer,
    tile_builder_impl::Builder,
};

pub use tile_builder_impl::ATLAS_SIZE;

pub struct Stage {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
    pub builder: Builder,
}

impl Stage {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
        let mut builder = Builder::new();

        let mut rasterizer = Rasterizer::new();
        builder.color = [152, 0, 152, 255];
        rasterizer.fill(
            &[
                PathCmd::Move(Vec2::new(400.0, 300.0)),
                PathCmd::Quadratic(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0)),
                PathCmd::Cubic(
                    Vec2::new(350.0, 150.0),
                    Vec2::new(100.0, 250.0),
                    Vec2::new(400.0, 300.0),
                ),
                PathCmd::Close,
            ],
            Transform::id(),
        );
        rasterizer.finish(&mut builder);

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&builder.vertices),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&builder.indices),
        );

        let tex = ctx.new_texture_from_data_and_format(
            &builder.atlas,
            TextureParams {
                width: ATLAS_SIZE as u32,
                height: ATLAS_SIZE as u32,
                format: TextureFormat::RGBA8,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                ..Default::default()
            },
        );
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![tex],
        };
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();
        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Short2),
                VertexAttribute::new("uv", VertexFormat::Short2),
                VertexAttribute::new("col", VertexFormat::Byte4),
            ],
            shader,
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
            pipeline,
            bindings,
            builder,
        }
    }
}

pub mod shader {
    use macroquad::miniquad::*;

    pub const VERTEX: &str = r#"
            #version 100
            precision highp float;

            uniform vec2 res;
            uniform vec2 atlas_size;

            attribute vec2 pos;
            attribute vec2 uv;
            attribute vec4 col;

            varying vec2 v_uv;
            varying vec4 v_col;

            void main() {
                vec2 scaled = 2.0 * pos / vec2(res);
                gl_Position = vec4(scaled.x - 1.0, 1.0 - scaled.y, 0.0, 1.0);
                v_uv = uv / vec2(atlas_size);
                v_col = col;
            }
        "#;

    pub const FRAGMENT: &str = r#"
            #version 100
            precision highp float;

            uniform sampler2D tex;

            varying vec2 v_uv;
            varying vec4 v_col;

            void main() {
                gl_FragColor = v_col * vec4(1.0, 1.0, 1.0, texture2D(tex, v_uv).r);
            }
        "#;

    pub const METAL: &str = r#"
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

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".into()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("res", UniformType::Float2),
                    UniformDesc::new("atlas_size", UniformType::Float2),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub res: [f32; 2],
        pub atlas_size: [f32; 2],
    }
}
