#[path = "../third_party/smaa/Textures/AreaTex.rs"]
mod area_tex;
use area_tex::*;

#[path = "../third_party/smaa/Textures/SearchTex.rs"]
mod search_tex;
use macroquad::{
    miniquad::{RenderingBackend, ShaderId, ShaderMeta, UniformBlockLayout},
    prelude::{ShaderError, ShaderSource},
};
use search_tex::*;

#[allow(dead_code)]
pub enum ShaderQuality {
    Low,
    Medium,
    High,
    Ultra,
}
impl ShaderQuality {
    fn as_str(&self) -> &'static str {
        match *self {
            ShaderQuality::Low => "LOW",
            ShaderQuality::Medium => "MEDIUM",
            ShaderQuality::High => "HIGH",
            ShaderQuality::Ultra => "ULTRA",
        }
    }
}

#[derive(Copy, Clone)]
pub enum ShaderStage {
    EdgeDetectionVS,
    LumaEdgeDetectionPS,

    BlendingWeightVS,
    BlendingWeightPS,

    NeighborhoodBlendingVS,
    NeighborhoodBlendingPS,

    #[allow(unused)]
    NeighborhoodBlendingAcesTonemapPS,
}
impl ShaderStage {
    fn is_vertex_shader(&self) -> bool {
        match *self {
            ShaderStage::EdgeDetectionVS
            | ShaderStage::BlendingWeightVS
            | ShaderStage::NeighborhoodBlendingVS => true,

            ShaderStage::LumaEdgeDetectionPS
            | ShaderStage::BlendingWeightPS
            | ShaderStage::NeighborhoodBlendingPS
            | ShaderStage::NeighborhoodBlendingAcesTonemapPS => false,
        }
    }

    fn as_str(&self) -> &'static str {
        match *self {
            ShaderStage::EdgeDetectionVS => {
                "varying vec4 offset0;
                 varying vec4 offset1;
                 varying vec4 offset2;
                 varying vec2 texcoord;

                 attribute vec2 in_pos;

                 void main() {
                     gl_Position = vec4(in_pos, 1.0, 1.0);
                     texcoord = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
                     vec4 offset[3];
                     SMAAEdgeDetectionVS(texcoord, offset);
                     offset0=offset[0];
                     offset1=offset[1];
                     offset2=offset[2];
                 }"
            }
            ShaderStage::LumaEdgeDetectionPS => {
                "varying vec4 offset0;
                 varying vec4 offset1;
                 varying vec4 offset2;
                 varying vec2 texcoord;
                 uniform sampler2D colorTex;
                 void main() {
                    float4 offset[3];
                    offset[0] = offset0;
                    offset[1] = offset1;
                    offset[2] = offset2;
                    gl_FragColor = vec4(SMAALumaEdgeDetectionPS(texcoord, offset, colorTex), 0.0, 1.0);
                 }"
            }
            ShaderStage::BlendingWeightVS => {
                "layout(location = 0) out float2 pixcoord;
                 layout(location = 1) out float4 offset0;
                 layout(location = 2) out float4 offset1;
                 layout(location = 3) out float4 offset2;
                 layout(location = 4) out float2 texcoord;
                 void main() {
                     if(gl_VertexIndex == 0) gl_Position = vec4(-1, -1, 1, 1);
                     if(gl_VertexIndex == 1) gl_Position = vec4(-1,  3, 1, 1);
        	         if(gl_VertexIndex == 2) gl_Position = vec4( 3, -1, 1, 1);
                     texcoord = gl_Position.xy * vec2(0.5, -0.5) + vec2(0.5);
                     float4 offset[3];
                     SMAABlendingWeightCalculationVS(texcoord, pixcoord, offset);
                     offset0=offset[0];
                     offset1=offset[1];
                     offset2=offset[2];
                 }"
            }
            ShaderStage::NeighborhoodBlendingVS => {
                "layout(location = 0) out float4 offset;
                 layout(location = 1) out float2 texcoord;
                 void main() {
                     if(gl_VertexIndex == 0) gl_Position = vec4(-1, -1, 1, 1);
                     if(gl_VertexIndex == 1) gl_Position = vec4(-1,  3, 1, 1);
        	         if(gl_VertexIndex == 2) gl_Position = vec4( 3, -1, 1, 1);
                     texcoord = gl_Position.xy * vec2(0.5, -0.5) + vec2(0.5);
                     SMAANeighborhoodBlendingVS(texcoord, offset);
                 }"
            }
            ShaderStage::BlendingWeightPS => {
                "layout(location = 0) in float2 pixcoord;
                 layout(location = 1) in float4 offset0;
                 layout(location = 2) in float4 offset1;
                 layout(location = 3) in float4 offset2;
                 layout(location = 4) in float2 texcoord;
                 layout(set = 0, binding = 2) uniform texture2D edgesTex;
                 layout(set = 0, binding = 3) uniform texture2D areaTex;
                 layout(set = 0, binding = 4) uniform texture2D searchTex;
                 layout(location = 0) out float4 OutColor;
                 void main() {
                     vec4 subsampleIndices = vec4(0);
                     float4 offset[3];
                     offset[0] = offset0;
                     offset[1] = offset1;
                     offset[2] = offset2;
                     OutColor = SMAABlendingWeightCalculationPS(texcoord, pixcoord, offset,
                         edgesTex, areaTex, searchTex, subsampleIndices);
                 }"
            }
            ShaderStage::NeighborhoodBlendingPS => {
                "layout(location = 0) in float4 offset;
                 layout(location = 1) in float2 texcoord;
                 layout(set = 0, binding = 2) uniform texture2D colorTex;
                 layout(set = 0, binding = 3) uniform texture2D blendTex;
                 layout(location = 0) out float4 OutColor;
                 void main() {
                     OutColor = SMAANeighborhoodBlendingPS(texcoord, offset, colorTex, blendTex);
                 }"
            }
            // See: https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve
            ShaderStage::NeighborhoodBlendingAcesTonemapPS => {
                "layout(location = 0) in float4 offset;
                 layout(location = 1) in float2 texcoord;
                 layout(set = 0, binding = 2) uniform texture2D colorTex;
                 layout(set = 0, binding = 3) uniform texture2D blendTex;
                 layout(location = 0) out float4 OutColor;
                 void main() {
                     float a = 2.51f;
                     float b = 0.03f;
                     float c = 2.43f;
                     float d = 0.59f;
                     float e = 0.14f;
                     OutColor = SMAANeighborhoodBlendingPS(texcoord, offset, colorTex, blendTex);
                     vec3 x = OutColor.rgb;
                     OutColor.rgb = clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3(0), vec3(1));
                 }"
            }
        }
    }
}

fn get_stage(quality: &ShaderQuality, stage: ShaderStage) -> String {
    format!(
        "#version 100
        precision lowp float;
        #define SMAA_GLSL_2
        #define SMAA_PRESET_{0}
        #define SMAA_INCLUDE_{1} 0
        #define SMAA_RT_METRICS u_rt
        uniform vec4 u_rt;
        {2}
        {3}",
        quality.as_str(),
        if stage.is_vertex_shader() { "PS" } else { "VS" },
        include_str!("../third_party/smaa/SMAA.hlsl"),
        stage.as_str(),
    )
}

pub fn get_shader(
    ctx: &mut dyn RenderingBackend,
    quality: ShaderQuality,
    vs_stage: ShaderStage,
    fs_stage: ShaderStage,
    meta: ShaderMeta,
) -> ShaderId {
    ctx.new_shader(
        ShaderSource::Glsl {
            vertex: &get_stage(&quality, vs_stage),
            fragment: &get_stage(&quality, fs_stage),
        },
        meta,
    )
    .unwrap()
}

pub mod raw_miniquad {
    use macroquad::miniquad::*;

    use super::{get_shader, ShaderQuality, ShaderStage};

    #[repr(C)]
    struct Vec2 {
        x: f32,
        y: f32,
    }

    #[repr(C)]
    struct Vertex {
        pos: [f32; 2],
        color: [f32; 4],
    }

    pub struct Stage {
        pub render_pipeline: Pipeline,
        pub render_bindings: Bindings,
        pub offscreen_pass: RenderPass,
        pub display_pipeline: Pipeline,
        pub display_bindings: Bindings,
        pub edge_detect_pipeline: Pipeline,
        pub edge_detect_bindings: Bindings,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
            #[rustfmt::skip]
            let vertices: [Vertex; 3] = [
                Vertex { pos : [ -0.5, -0.5 ], color: [1., 0., 0., 1.] },
                Vertex { pos : [  0.5, -0.5 ], color: [0., 1., 0., 1.] },
                Vertex { pos : [  0.0,  0.5 ], color: [0., 0., 1., 1.] },
            ];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );

            let indices: [u16; 3] = [0, 1, 2];
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );

            let render_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer: index_buffer,
                images: vec![],
            };

            let shader = ctx
                .new_shader(
                    ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    shader::meta(),
                )
                .unwrap();

            let render_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("in_pos", VertexFormat::Float2),
                    VertexAttribute::new("in_color", VertexFormat::Float4),
                ],
                shader,
                PipelineParams::default(),
            );

            let color_img = ctx.new_render_texture(TextureParams {
                width: 0,
                height: 0,
                format: TextureFormat::RGBA8,
                ..Default::default()
            });
            let depth_img = ctx.new_render_texture(TextureParams {
                width: 0,
                height: 0,
                format: TextureFormat::Depth,
                ..Default::default()
            });

            let offscreen_pass = ctx.new_render_pass(color_img, Some(depth_img));

            let vertices: [[f32; 2]; 6] = [
                [-1., -1.],
                [-1., 1.],
                [1., 1.],
                [1., 1.],
                [1., -1.],
                [-1., -1.],
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

            let display_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![color_img],
            };

            let display_shader = ctx
                .new_shader(
                    ShaderSource::Glsl {
                        vertex: shader::DISPLAY_VERTEX,
                        fragment: shader::DISPLAY_FRAGMENT,
                    },
                    ShaderMeta {
                        images: vec!["tex".to_string()],
                        uniforms: UniformBlockLayout { uniforms: vec![] },
                    },
                )
                .unwrap();

            let display_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("in_pos", VertexFormat::Float2)],
                display_shader,
                PipelineParams::default(),
            );

            let vertices: [[f32; 2]; 3] = [[-1., -1.], [-1., 3.], [3., -1.]];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );

            let indices: [u16; 3] = [0, 1, 2];
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );

            let edge_detect_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![color_img],
            };

            let shader = get_shader(
                ctx,
                ShaderQuality::High,
                ShaderStage::EdgeDetectionVS,
                ShaderStage::LumaEdgeDetectionPS,
                ShaderMeta {
                    images: vec!["colorTex".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![UniformDesc::new("u_rt", UniformType::Float4)],
                    },
                },
            );
            let edge_detect_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("in_pos", VertexFormat::Float2)],
                shader,
                PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
                        BlendFactor::Zero,
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
                        BlendFactor::Zero,
                    )),
                    ..Default::default()
                },
            );

            Stage {
                render_pipeline,
                render_bindings,
                offscreen_pass,
                display_pipeline,
                display_bindings,
                edge_detect_pipeline,
                edge_detect_bindings,
            }
        }
    }

    pub mod shader {
        use macroquad::miniquad::*;

        pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;

    varying lowp vec4 color;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        color = in_color;
    }"#;

        pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }"#;

        pub fn meta() -> ShaderMeta {
            ShaderMeta {
                images: vec![],
                uniforms: UniformBlockLayout { uniforms: vec![] },
            }
        }

        pub const DISPLAY_VERTEX: &str = r#"
            #version 100
            attribute vec2 in_pos;

            void main() {
                gl_Position = vec4(in_pos, 0, 1);
            }"#;

        pub const DISPLAY_FRAGMENT: &str = r#"
            #version 100
            uniform sampler2D tex;

            void main() {
                gl_FragColor = vec4(texture2D(tex, vec2(gl_FragCoord.x / 800.0, gl_FragCoord.y / 600.0)).xyz, 1.0);
            }"#;

        #[repr(C)]
        pub struct Uniforms {
            pub u_rt: [f32; 4],
        }
    }
}
