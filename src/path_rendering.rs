// Most of it copied from https://github.com/micahrj/ochre

mod geom;
mod path;
mod rasterizer;
mod tile_builder_impl;

use {
    geom::{Mat2x2, Transform, Vec2},
    macroquad::miniquad::*,
    path::PathCmd,
    rasterizer::Rasterizer,
    tile_builder_impl::Builder,
    tiny_skia_path::{PathSegment, Point},
};

pub use tile_builder_impl::ATLAS_SIZE;

pub struct Stage {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
    pub builder: Builder,
}

impl Stage {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
        let svg_data = include_bytes!("../res/Ghostscript_Tiger.svg");
        let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default()).unwrap();
        let mut builder = Builder::new();

        fn render_node(node: &usvg::Node, builder: &mut Builder) {
            match node {
                usvg::Node::Path(ref p) => {
                    let t = node.abs_transform();
                    let transform = Transform::new(
                        Mat2x2::new(t.sx as f32, t.ky as f32, t.kx as f32, t.sy as f32),
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
                            PathSegment::QuadTo(
                                Point { x: x1, y: y1 },
                                Point { x: x2, y: y2 },
                            ) => {
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
                            builder.color =
                                [color.red, color.green, color.blue, f.opacity().to_u8()];
                            let mut rasterizer = Rasterizer::new();
                            rasterizer.fill(&path, transform);
                            rasterizer.finish(builder);
                        }
                    }

                    if let Some(ref s) = p.stroke() {
                        if let usvg::Paint::Color(color) = s.paint() {
                            builder.color =
                                [color.red, color.green, color.blue, s.opacity().to_u8()];
                            let mut rasterizer = Rasterizer::new();
                            rasterizer.stroke(&path, s.width().get() as f32, transform);
                            rasterizer.finish(builder);
                        }
                    }
                }
                usvg::Node::Group(ref g) => {
                    render_nodes(g, builder);
                }
                _ => {}
            }
        }

        fn render_nodes(group: &usvg::Group, builder: &mut Builder) {
            for child in group.children() {
                render_node(&child, builder);
            }
        }

        // render_nodes(&tree.root(), &mut builder);

        // let mut rasterizer = Rasterizer::new();
        // builder.color = [152, 0, 152, 255];
        // rasterizer.fill(
        //     &[
        //         PathCmd::Move(Vec2::new(200.0, 300.0)),
        //         PathCmd::Quadratic(Vec2::new(300.0, 200.0), Vec2::new(200.0, 100.0)),
        //         PathCmd::Cubic(
        //             Vec2::new(150.0, 150.0),
        //             Vec2::new(-100.0, 250.0),
        //             Vec2::new(200.0, 300.0),
        //         ),
        //         PathCmd::Close,
        //     ],
        //     Transform::id(),
        // );
        // rasterizer.finish(&mut builder);

        let mut rasterizer = Rasterizer::new();

        // Let's say it's circle
        builder.color = [15, 201, 52, 255];
        rasterizer.fill(
            &[
                PathCmd::Move(Vec2::new(200.0, 200.0)),
                PathCmd::Quadratic(Vec2::new(250.0, 200.0), Vec2::new(250.0, 250.0)),
                PathCmd::Quadratic(Vec2::new(250.0, 300.0), Vec2::new(200.0, 300.0)),
                PathCmd::Quadratic(Vec2::new(150.0, 300.0), Vec2::new(150.0, 250.0)),
                PathCmd::Quadratic(Vec2::new(150.0, 200.0), Vec2::new(200.0, 200.0)),
                PathCmd::Close,
            ],
            Transform::id(),
        );
        rasterizer.finish(&mut builder);

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&builder.vertices),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
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
                v_col = col / 255.0;
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
            float2 res;
            float2 atlas_size;
        };

        struct Vertex
        {
            short2 pos  [[attribute(0)]];
            short2 uv   [[attribute(1)]];
            uchar4 col  [[attribute(2)]];
        };

        struct RasterizerData
        {
            float4 position [[position]];
            float2 uv [[user(loc0)]];
            float4 col [[user(loc1)]];
        };

        vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
        {
            RasterizerData out;

            float2 scaled = 2.0 * float2(v.pos) / float2(uniforms.res);
            out.position = float4(scaled.x - 1.0, 1.0 - scaled.y, 0.0, 1.0);
            out.uv = float2(v.uv) / float2(uniforms.atlas_size);
            out.col = float4(v.col) / 255.0;

            return out;
        }

        fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                                       texture2d<float> tex [[texture(0)]],
                                       sampler smplr [[sampler(0)]])
        {
            return in.col * float4(1.0, 1.0, 1.0, tex.sample(smplr, in.uv).r);
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
