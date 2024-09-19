// Most of it copied from https://github.com/Lichtso/contrast_renderer
mod error;
mod path;
mod safe_float;
pub mod utils;

use {
    error::{Error, ERROR_MARGIN},
    geometric_algebra::RegressiveProduct,
    path::{Path, SegmentType},
    safe_float::SafeFloat,
    utils::vec_to_point,
};

pub type Vertex0 = [f32; 2];

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct Vertex3f(pub [f32; 2], pub [f32; 3]);

#[derive(Default)]
pub struct FillBuilder {
    pub solid_indices: Vec<u16>,
    pub solid_vertices: Vec<Vertex0>,
    pub rational_quadratic_vertices: Vec<Vertex3f>,
}

impl FillBuilder {
    pub fn add_path(
        &mut self,
        proto_hull: &mut Vec<SafeFloat<f32, 2>>,
        path: &Path,
    ) -> Result<(), Error> {
        let mut path_solid_vertices: Vec<Vertex0> = Vec::with_capacity(
            1 + path.line_segments.len()
                + path.integral_quadratic_curve_segments.len()
                + path.integral_cubic_curve_segments.len() * 5
                + path.rational_quadratic_curve_segments.len()
                + path.rational_cubic_curve_segments.len() * 5,
        );
        path_solid_vertices.push(path.start.unwrap());
        proto_hull.push(path.start);
        let mut rational_quadratic_curve_segment_iter =
            path.rational_quadratic_curve_segments.iter();
        for segment_type in &path.segment_types {
            match segment_type {
                SegmentType::RationalQuadraticCurve => {
                    let segment = rational_quadratic_curve_segment_iter.next().unwrap();
                    let weight = 1.0 / segment.weight.unwrap();
                    self.rational_quadratic_vertices.push(Vertex3f(
                        segment.control_points[1].unwrap(),
                        [1.0, 1.0, 1.0],
                    ));
                    self.rational_quadratic_vertices.push(Vertex3f(
                        segment.control_points[0].unwrap(),
                        [0.5 * weight, 0.0, weight],
                    ));
                    self.rational_quadratic_vertices.push(Vertex3f(
                        *path_solid_vertices.last().unwrap(),
                        [0.0, 0.0, 1.0],
                    ));
                    proto_hull.push(segment.control_points[0]);
                    proto_hull.push(segment.control_points[1]);
                    path_solid_vertices.push(segment.control_points[1].unwrap());
                }
                _ => (),
            }
        }
        let start_index = self.solid_vertices.len();
        self.solid_vertices
            .append(&mut triangle_fan_to_strip(path_solid_vertices));
        let mut indices: Vec<u16> =
            (start_index as u16..(self.solid_vertices.len() + 1) as u16).collect();
        *indices.iter_mut().last().unwrap() = (-1isize) as u16;
        self.solid_indices.append(&mut indices);
        Ok(())
    }
}

/// Concats the given sequence of [Buffer]s and serializes them into a `Vec<u8>`
#[macro_export]
macro_rules! concat_buffers {
    (count: $buffer:expr $(,)?) => {
        1
    };
    (count: $buffer:expr, $($rest:expr),+) => {
        concat_buffers!(count: $($rest),*) + 1
    };
    ([$($buffer:expr),* $(,)?]) => {{
        let buffers = [
            $(utils::transmute_slice::<_, u8>($buffer)),*
        ];
        let mut end_offsets = [0; $crate::concat_buffers!(count: $($buffer),*)];
        let mut buffer_length = 0;
        for (i, buffer) in buffers.iter().enumerate() {
            buffer_length += buffer.len();
            end_offsets[i] = buffer_length;
        }
        let buffer_data = buffers.concat();
        (end_offsets, buffer_data)
    }};
}

/// A set of [Path]s which is always rendered together
pub struct RenderShape {
    pub vertex_offsets: [usize; 3],
    pub index_offsets: [usize; 1],
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u8>,
}

impl RenderShape {
    fn from_paths(paths: &[Path]) -> Result<Self, Error> {
        let mut proto_hull = Vec::new();
        let mut fill_builder = FillBuilder::default();
        for path in paths {
            fill_builder.add_path(&mut proto_hull, path)?;
        }
        let convex_hull = triangle_fan_to_strip(andrew(&proto_hull));
        dbg!(&fill_builder.solid_vertices);
        let (vertex_offsets, vertex_buffer) = concat_buffers!([
            &fill_builder.solid_vertices,
            &fill_builder.rational_quadratic_vertices,
            &convex_hull,
        ]);
        dbg!(&fill_builder.solid_indices);
        dbg!(&fill_builder.rational_quadratic_vertices);
        let (index_offsets, index_buffer) = concat_buffers!([&fill_builder.solid_indices]);

        Ok(Self {
            vertex_offsets,
            index_offsets,
            vertex_buffer,
            index_buffer,
        })
    }
}

/// Andrew's (monotone chain) convex hull algorithm
pub fn andrew(input_points: &[SafeFloat<f32, 2>]) -> Vec<[f32; 2]> {
    let mut input_points = input_points.to_owned();
    if input_points.len() < 3 {
        return input_points
            .iter()
            .map(|input_point| input_point.unwrap())
            .collect();
    }
    input_points.sort();
    let mut hull = Vec::with_capacity(2 * input_points.len());
    for input_point in input_points.iter().cloned() {
        while hull.len() > 1
            && vec_to_point(hull[hull.len() - 2])
                .regressive_product(vec_to_point(hull[hull.len() - 1]))
                .regressive_product(vec_to_point(input_point.unwrap()))
                <= ERROR_MARGIN
        {
            hull.pop();
        }
        hull.push(input_point.unwrap());
    }
    hull.pop();
    let t = hull.len() + 1;
    for input_point in input_points.iter().rev().cloned() {
        while hull.len() > t
            && vec_to_point(hull[hull.len() - 2])
                .regressive_product(vec_to_point(hull[hull.len() - 1]))
                .regressive_product(vec_to_point(input_point.unwrap()))
                <= ERROR_MARGIN
        {
            hull.pop();
        }
        hull.push(input_point.unwrap());
    }
    hull.pop();
    hull
}

pub fn triangle_fan_to_strip<T: Copy>(vertices: Vec<T>) -> Vec<T> {
    let gather_indices = (0..vertices.len()).map(|i| {
        if (i & 1) == 0 {
            i >> 1
        } else {
            vertices.len() - 1 - (i >> 1)
        }
    });
    let mut result = Vec::with_capacity(vertices.len());
    for src in gather_indices {
        result.push(vertices[src]);
    }
    result
}

pub mod raw_miniquad {
    use super::path::Path;
    use super::RenderShape;
    use miniquad::*;

    pub struct Stage {
        pub fill_solid_pipeline: Pipeline,
        pub fill_solid_bindings: Bindings,
        pub fill_rational_quadratic_curve_pipeline: Pipeline,
        pub fill_rational_quadratic_curve_bindings: Bindings,
        pub shape2: RenderShape,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
            let shape2 =
                RenderShape::from_paths(&vec![Path::from_circle([0.0, 0.0], 0.5)]).unwrap();

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
            dbg!(&indices);

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
                    miniquad::ShaderSource::Glsl {
                        vertex: shader::FILL_VERTEX,
                        fragment: shader::FILL_FRAGMENT,
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

            // dbg!(shape2.vertex_offsets);
            // dbg!(&shape2.vertex_buffer);
            // dbg!(shape2.index_offsets);
            // dbg!(&shape2.index_buffer);

            let begin_offset: usize = shape2.vertex_offsets[0];
            let end_offset = shape2.vertex_offsets[1];
            let vertices = &shape2.vertex_buffer[begin_offset..end_offset];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(vertices),
            );

            let vertex_size = std::mem::size_of::<super::Vertex3f>();
            dbg!(vertex_size);
            let indices: Vec<u16> =
                (0..((end_offset - begin_offset) / vertex_size) as u16).collect();
            dbg!(&indices);

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
                    miniquad::ShaderSource::Glsl {
                        vertex: shader::QUADRATIC_VERTEX,
                        fragment: shader::QUADRATIC_FRAGMENT,
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
