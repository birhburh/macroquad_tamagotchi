//! Rendering of [Path]s bundeled in [Shape]s using a [Renderer]

use {
    super::{
        error::{Error, ERROR_MARGIN},
        fill::FillBuilder,
        path::Path,
        safe_float::SafeFloat,
        utils::{transmute_slice, vec_to_point},
        vertex::triangle_fan_to_triangles,
    },
    geometric_algebra::RegressiveProduct,
};

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
            $(transmute_slice::<_, u8>($buffer)),*
        ];
        let mut end_offsets = [0; concat_buffers!(count: $($buffer),*)];
        let mut buffer_length = 0;
        for (i, buffer) in buffers.iter().enumerate() {
            buffer_length += buffer.len();
            end_offsets[i] = buffer_length;
        }
        let buffer_data = buffers.concat();
        (end_offsets, buffer_data)
    }};
}

/// Which shader to use for rendering a shape
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum RenderOperation {
    /// Prepare rendering the [Shape]
    Stencil,
    /// Start using the rendered [Shape] as stencil for other [Shape]s
    Clip,
    /// Stop using the rendered [Shape] as stencil for other [Shape]s
    UnClip,
    /// Render the [Shape] as a solid color using alpha blending
    Color,
    /// Start using the rendered [Shape] as opacity group for other [Shape]s
    SaveAlphaContext,
    /// Second step of [RenderOperation::SaveAlphaContext], needs its own [wgpu::RenderPass]
    ScaleAlphaContext,
    /// Stop using the rendered [Shape] as opacity group for other [Shape]s
    RestoreAlphaContext,
}

/// A set of [Path]s which is always rendered together
pub struct Shape {
    pub vertex_offsets: [usize; 7],
    pub index_offsets: [usize; 1],
    pub vertex_buffer: Vec<u8>,
    pub index_buffer: Vec<u8>,
    pub convex_box: Vec<f32>,
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

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
}

impl Shape {
    pub fn from_paths(paths: &[Path]) -> Result<Self, Error> {
        let mut proto_hull = Vec::new();
        let mut fill_builder = FillBuilder::default();
        for path in paths {
            fill_builder.add_path(&mut proto_hull, path)?;
        }
        let convex_hull = triangle_fan_to_triangles(andrew(&proto_hull));
        let mut convex_box = vec![
            convex_hull[0][0],
            convex_hull[0][1],
            convex_hull[0][0],
            convex_hull[0][1],
        ];
        for point in &convex_hull {
            if point[0] < convex_box[0] {
                convex_box[0] = point[0];
            }
            if point[0] > convex_box[2] {
                convex_box[2] = point[0];
            }
            if point[1] < convex_box[1] {
                convex_box[1] = point[1];
            }
            if point[1] > convex_box[3] {
                convex_box[3] = point[1];
            }
        }
        dbg!(&convex_box);
        let full_screen_texture: [Vertex; 6] = [
            Vertex {
                pos: Vec2 { x: -1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1., y: -1. },
            },
            Vertex {
                pos: Vec2 { x: 1., y: 1. },
            },
            Vertex {
                pos: Vec2 { x: -1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1.0, y: 1.0 },
            },
            Vertex {
                pos: Vec2 { x: -1.0, y: 1.0 },
            },
        ];
        let (vertex_offsets, vertex_buffer) = concat_buffers!([
            &fill_builder.solid_vertices,
            &fill_builder.integral_quadratic_vertices,
            &fill_builder.integral_cubic_vertices,
            &fill_builder.rational_quadratic_vertices,
            &fill_builder.rational_cubic_vertices,
            &convex_hull,
            &full_screen_texture
        ]);
        let (index_offsets, index_buffer) = concat_buffers!([&fill_builder.solid_indices]);

        Ok(Self {
            vertex_offsets,
            index_offsets,
            vertex_buffer,
            index_buffer,
            convex_box,
        })
    }
}
