// Just for backup
use miniquad::*;
use window::screen_size;

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 6] = [
            Vertex { pos : [ -1.0, -1.0 ] },
            Vertex { pos : [ -1.0,  1.0 ] },
            Vertex { pos : [  1.0,  1.0 ] },
            Vertex { pos : [  1.0,  1.0 ] },
            Vertex { pos : [  1.0, -1.0 ] },
            Vertex { pos : [ -1.0, -1.0 ] },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 3, 4, 5];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
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
                VertexAttribute::new("in_pos", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        Stage {
            pipeline,
            bindings,
            ctx,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms {
                i_resolution: screen_size(),
            }));
        self.ctx.draw(0, 6, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;
    uniform vec2 iResolution;

    void main() {
        vec2 uv = gl_FragCoord.xy/iResolution.xy * 2. - 1.;
        uv.x *= iResolution.x / iResolution.y;

        vec2 position = vec2(.0, 0.);
        float radius = .7;
        float width = .1;
        float dist = distance(uv, position);
        vec4 color = vec4(.8, .7, 1., 1.);

        float aa = 2.0 / min(iResolution.x, iResolution.y);

        // Our alpha channel
        float alpha = 1. - smoothstep(width - aa, width + aa, abs(dist - radius));

        color.a *= alpha;
        color.rgb *= alpha;

        gl_FragColor = color;
    }
    "#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
    };

    struct Uniforms
    {
        float2 iResolution;
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy, 0.0, 1.0);

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                                   constant Uniforms& uniforms [[buffer(0)]])
    {
        float2 uv = in.position.xy/uniforms.iResolution.xy * 2. - 1.;
        uv.x *= uniforms.iResolution.x / uniforms.iResolution.y;

        float2 position = float2(.0, 0.);
        float radius = .7;
        float width = .1;
        float dist = distance(uv, position);
        float4 color = float4(.8, .7, 1., 1.);

        float aa = 2.0 / min(uniforms.iResolution.x, uniforms.iResolution.y);

        // Our alpha channel
        float alpha = 1. - smoothstep(width - aa, width + aa, abs(dist - radius));

        color.a *= alpha;
        color.rgb *= alpha;

        return color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub i_resolution: (f32, f32),
    }
}
