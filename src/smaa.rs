#[path = "../third_party/smaa/Textures/AreaTex.rs"]
mod area_tex;
use area_tex::*;

#[path = "../third_party/smaa/Textures/SearchTex.rs"]
mod search_tex;
use macroquad::{
    miniquad::{RenderingBackend, ShaderId, ShaderMeta},
    prelude::ShaderSource,
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
}
impl ShaderStage {
    fn is_vertex_shader(&self) -> bool {
        match *self {
            ShaderStage::EdgeDetectionVS
            | ShaderStage::BlendingWeightVS
            | ShaderStage::NeighborhoodBlendingVS => true,

            ShaderStage::LumaEdgeDetectionPS
            | ShaderStage::BlendingWeightPS
            | ShaderStage::NeighborhoodBlendingPS => false,
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
                    vec4 offset[3];
                    offset[0] = offset0;
                    offset[1] = offset1;
                    offset[2] = offset2;
                    gl_FragColor = vec4(SMAALumaEdgeDetectionPS(texcoord, offset, colorTex), 0.0, 1.0);
                 }"
            }
            ShaderStage::BlendingWeightVS => {
                "varying vec2 pixcoord;
                 varying vec4 offset0;
                 varying vec4 offset1;
                 varying vec4 offset2;
                 varying vec2 texcoord;

                 attribute vec2 in_pos;

                 void main() {
                     gl_Position = vec4(in_pos, 1.0, 1.0);
                     texcoord = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
                     vec4 offset[3];
                     SMAABlendingWeightCalculationVS(texcoord, pixcoord, offset);
                     offset0=offset[0];
                     offset1=offset[1];
                     offset2=offset[2];
                 }"
            }
            ShaderStage::BlendingWeightPS => {
                "varying vec2 pixcoord;
                 varying vec4 offset0;
                 varying vec4 offset1;
                 varying vec4 offset2;
                 varying vec2 texcoord;
                 uniform sampler2D edgesTex;
                 uniform sampler2D areaTex;
                 uniform sampler2D searchTex;
                 void main() {
                     vec4 subsampleIndices = vec4(0);
                     vec4 offset[3];
                     offset[0] = offset0;
                     offset[1] = offset1;
                     offset[2] = offset2;
                     gl_FragColor = SMAABlendingWeightCalculationPS(texcoord, pixcoord, offset,
                         edgesTex, areaTex, searchTex, subsampleIndices);
                 }"
            }
            ShaderStage::NeighborhoodBlendingVS => {
                "varying vec4 offset;
                 varying vec2 texcoord;

                 attribute vec2 in_pos;

                 void main() {
                     gl_Position = vec4(in_pos, 1.0, 1.0);
                     texcoord = gl_Position.xy * vec2(0.5, 0.5) + vec2(0.5);
                     SMAANeighborhoodBlendingVS(texcoord, offset);
                 }"
            }
            ShaderStage::NeighborhoodBlendingPS => {
                "varying vec4 offset;
                 varying vec2 texcoord;
                 uniform sampler2D colorTex;
                 uniform sampler2D blendTex;
                 void main() {
                     gl_FragColor = SMAANeighborhoodBlendingPS(texcoord, offset, colorTex, blendTex);
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

    use super::{
        get_shader, ShaderQuality, ShaderStage, AREATEX_BYTES, AREATEX_HEIGHT, AREATEX_WIDTH,
        SEARCHTEX_BYTES, SEARCHTEX_HEIGHT, SEARCHTEX_WIDTH,
    };

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
        pub render_offscreen_pass: RenderPass,
        pub edge_detect_pipeline: Pipeline,
        pub edge_detect_bindings: Bindings,
        pub edge_detect_offscreen_pass: RenderPass,
        pub blend_weight_pipeline: Pipeline,
        pub blend_weight_bindings: Bindings,
        pub blend_weight_offscreen_pass: RenderPass,
        pub neighborhood_blending_pipeline: Pipeline,
        pub neighborhood_blending_bindings: Bindings,
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

            let render_offscreen_pass = ctx.new_render_pass(color_img, None);

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
                    // color_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    // alpha_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    ..Default::default()
                },
            );

            let edge_detect_offscreen_pass = ctx.new_render_pass(color_img, None);
            let area_img = ctx.new_texture_from_data_and_format(
                &AREATEX_BYTES,
                TextureParams {
                    kind: TextureKind::Texture2D,
                    width: AREATEX_WIDTH,
                    height: AREATEX_HEIGHT,
                    format: TextureFormat::RGBA8,
                    wrap: TextureWrap::Clamp,
                    min_filter: FilterMode::Linear,
                    mag_filter: FilterMode::Linear,
                    mipmap_filter: MipmapFilterMode::None,
                    allocate_mipmaps: false,
                    sample_count: 0,
                },
            );
            let search_img = ctx.new_texture_from_data_and_format(
                &SEARCHTEX_BYTES,
                TextureParams {
                    kind: TextureKind::Texture2D,
                    width: SEARCHTEX_WIDTH,
                    height: SEARCHTEX_HEIGHT,
                    format: TextureFormat::RGBA8,
                    wrap: TextureWrap::Clamp,
                    min_filter: FilterMode::Linear,
                    mag_filter: FilterMode::Linear,
                    mipmap_filter: MipmapFilterMode::None,
                    allocate_mipmaps: false,
                    sample_count: 0,
                },
            );
            let blend_weight_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![color_img, area_img, search_img],
            };

            let shader = get_shader(
                ctx,
                ShaderQuality::High,
                ShaderStage::BlendingWeightVS,
                ShaderStage::BlendingWeightPS,
                ShaderMeta {
                    images: vec![
                        "edgesTex".to_string(),
                        "areaTex".to_string(),
                        "searchTex".to_string(),
                    ],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![UniformDesc::new("u_rt", UniformType::Float4)],
                    },
                },
            );
            let blend_weight_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("in_pos", VertexFormat::Float2)],
                shader,
                PipelineParams {
                    // color_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    // alpha_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    ..Default::default()
                },
            );
            let blend_weight_offscreen_pass = ctx.new_render_pass(color_img, None);

            let neighborhood_blending_bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![color_img, color_img],
            };

            let shader = get_shader(
                ctx,
                ShaderQuality::High,
                ShaderStage::NeighborhoodBlendingVS,
                ShaderStage::NeighborhoodBlendingPS,
                ShaderMeta {
                    images: vec!["colorTex".to_string(), "blendTex".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![UniformDesc::new("u_rt", UniformType::Float4)],
                    },
                },
            );
            let neighborhood_blending_pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[VertexAttribute::new("in_pos", VertexFormat::Float2)],
                shader,
                PipelineParams {
                    // color_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    // alpha_blend: Some(BlendState::new(
                    //     Equation::Add,
                    //     BlendFactor::One,
                    //     BlendFactor::Zero,
                    // )),
                    ..Default::default()
                },
            );

            Stage {
                render_pipeline,
                render_bindings,
                render_offscreen_pass,
                edge_detect_pipeline,
                edge_detect_bindings,
                edge_detect_offscreen_pass,
                blend_weight_pipeline,
                blend_weight_bindings,
                blend_weight_offscreen_pass,
                neighborhood_blending_pipeline,
                neighborhood_blending_bindings,
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
