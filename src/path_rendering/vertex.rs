pub type Vertex0 = [f32; 2];

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct Vertex2f(pub [f32; 2], pub [f32; 2]);

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct Vertex2f1i(pub [f32; 2], pub [f32; 2], pub u32);

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct Vertex3f(pub [f32; 2], pub [f32; 3]);

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct Vertex3f1i(pub [f32; 2], pub [f32; 3], pub u32);

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct Vertex4f(pub [f32; 2], pub [f32; 4]);

pub fn triangle_fan_to_triangles<T: Copy + std::fmt::Debug>(vertices: Vec<T>) -> Vec<T> {
    // vertices order:
    // 0 1 2 3
    // new order:
    // 0 1 2 3 4 5
    // 0 2 1 0 3 2
    // if more vertices it looks like this:
    // 0 1 2 3 4 5 6 7 8 9 10 11
    // 0 2 1 0 3 2 0 4 3 0  5  4
    let gather_indices = (0..((vertices.len() - 2) * 3)).map(|i| match i % 3 {
        0 => 0,
        1 => (i / 3) + 2,
        2 => (i / 3) + 1,
        _ => unreachable!(),
    });

    let mut result = Vec::with_capacity((vertices.len() - 2) * 3);
    for src in gather_indices {
        result.push(vertices[src]);
    }
    result
}
