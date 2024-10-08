use super::{
    curve::{
        inflection_point_polynomial_coefficients, integral_inflection_points,
        rational_cubic_control_points_to_power_basis, rational_cubic_first_order_derivative,
        rational_inflection_points,
    },
    error::{Error, ERROR_MARGIN},
    path::{Path, SegmentType},
    safe_float::SafeFloat,
    utils::{point_to_vec, vec_to_point, weighted_vec_to_point},
    vertex::{triangle_fan_to_triangles, Vertex0, Vertex2f, Vertex3f, Vertex4f},
};
use geometric_algebra::{
    polynomial::Root, ppga2d, ppga3d, InnerProduct, RegressiveProduct, SquaredMagnitude, Zero,
};

fn find_double_point_issue(discriminant: f32, roots: &[Root; 3]) -> Option<f32> {
    if discriminant < 0.0 {
        let mut result = -1.0;
        let mut inside = 0;
        for root in roots {
            if root.denominator != 0.0 {
                let parameter = root.numerator.real() / root.denominator;
                if 0.0 < parameter && parameter < 1.0 {
                    result = parameter;
                    inside += 1;
                }
            }
        }
        if inside == 1 {
            return Some(result);
        }
    }
    None
}

fn weight_derivatives(weights: &mut [[f32; 4]; 4], column: usize, roots: [Root; 3]) {
    let power_basis = [
        roots[0].numerator.real() * roots[1].numerator.real() * roots[2].numerator.real(),
        -roots[0].denominator * roots[1].numerator.real() * roots[2].numerator.real()
            - roots[0].numerator.real() * roots[1].denominator * roots[2].numerator.real()
            - roots[0].numerator.real() * roots[1].numerator.real() * roots[2].denominator,
        roots[0].numerator.real() * roots[1].denominator * roots[2].denominator
            + roots[0].denominator * roots[1].numerator.real() * roots[2].denominator
            + roots[0].denominator * roots[1].denominator * roots[2].numerator.real(),
        -roots[0].denominator * roots[1].denominator * roots[2].denominator,
    ];
    weights[0][column] = power_basis[0];
    weights[1][column] = power_basis[0] + power_basis[1] * 1.0 / 3.0;
    weights[2][column] = power_basis[0] + power_basis[1] * 2.0 / 3.0 + power_basis[2] * 1.0 / 3.0;
    weights[3][column] = power_basis[0] + power_basis[1] + power_basis[2] + power_basis[3];
}

fn weights(discriminant: f32, roots: &[Root; 3]) -> [[f32; 4]; 4] {
    let mut weights = [[0.0; 4]; 4];
    if discriminant == 0.0 {
        weight_derivatives(&mut weights, 0, [roots[0], roots[0], roots[2]]);
        weight_derivatives(&mut weights, 1, [roots[0], roots[0], roots[0]]);
        weight_derivatives(&mut weights, 2, [roots[0], roots[0], roots[0]]);
    } else if discriminant < 0.0 {
        weight_derivatives(&mut weights, 0, [roots[0], roots[1], roots[2]]);
        weight_derivatives(&mut weights, 1, [roots[0], roots[0], roots[1]]);
        weight_derivatives(&mut weights, 2, [roots[1], roots[1], roots[0]]);
    } else {
        weight_derivatives(&mut weights, 0, [roots[0], roots[1], roots[2]]);
        weight_derivatives(&mut weights, 1, [roots[0], roots[0], roots[0]]);
        weight_derivatives(&mut weights, 2, [roots[1], roots[1], roots[1]]);
    }
    weight_derivatives(&mut weights, 3, [roots[2], roots[2], roots[2]]);
    weights
}

fn weight_planes(
    control_points: &[ppga2d::Point; 4],
    weights: &[[f32; 4]; 4],
) -> [ppga2d::Plane; 4] {
    let mut planes = [ppga2d::Plane::zero(); 4];
    let mut points = [ppga3d::Point::zero(); 4];
    for (i, plane_2d) in planes.iter_mut().enumerate() {
        for (j, control_point) in control_points.iter().enumerate() {
            points[j] = ppga3d::Point::from([
                control_point[0],
                control_point[1],
                control_point[2],
                weights[j][i],
            ]);
        }
        let mut plane_3d = points[0]
            .regressive_product(points[1])
            .regressive_product(points[2]);
        if plane_3d.squared_magnitude() < ERROR_MARGIN {
            plane_3d = points[0]
                .regressive_product(points[1])
                .regressive_product(points[3]);
        }
        plane_3d = plane_3d * (1.0 / -plane_3d[3]);
        *plane_2d = ppga2d::Plane::new(plane_3d[0], plane_3d[1], plane_3d[2]);
    }
    planes
}

fn implicit_curve_value(weights: ppga3d::Point) -> f32 {
    weights[0].powi(3) - weights[1] * weights[2] * weights[3]
}

fn implicit_curve_gradient(planes: &[ppga2d::Plane; 4], weights: &[f32; 4]) -> ppga2d::Plane {
    planes[0] * (3.0 * weights[0] * weights[0])
        - planes[1] * (weights[2] * weights[3])
        - planes[2] * (weights[1] * weights[3])
        - planes[3] * (weights[1] * weights[2])
}

fn normalize_implicit_curve_side(
    planes: &mut [ppga2d::Plane; 4],
    weights: &mut [[f32; 4]; 4],
    power_basis: &[ppga2d::Point; 4],
    gradient: ppga2d::Plane,
) {
    let tangent = rational_cubic_first_order_derivative(power_basis, 0.0);
    if tangent.inner_product(gradient) > 0.0 {
        for plane in planes {
            *plane = -*plane;
        }
        for row in weights {
            row[0] *= -1.0;
            row[1] *= -1.0;
        }
    }
}

macro_rules! emit_cubic_curve_triangle {
    ($triangles:expr, $signed_triangle_areas:expr, $control_points:expr, $weights:expr, $v:ident, $w:ident, $emit_vertex:expr, $triangle_index:expr) => {
        let mut triangle = Vec::new();
        for vertex_index in (0..4).filter(|i| *i != $triangle_index) {
            let $v = point_to_vec($control_points[vertex_index]);
            let $w = $weights[vertex_index];
            triangle.push($emit_vertex);
        }
        let signed_triangle_area = $signed_triangle_areas[$triangle_index];
        if signed_triangle_area.abs() > ERROR_MARGIN {
            if signed_triangle_area < 0.0 {
                triangle.reverse();
            }
            $triangles.append(&mut triangle);
        }
    };
}

macro_rules! triangulate_cubic_curve_quadrilateral {
    ($fill_solid_vertices:expr, $cubic_vertices:expr,
     $control_points:expr, $weights:expr, $v:ident, $w:ident, $emit_vertex:expr) => {{
        for (weights, control_point) in $weights.iter_mut().zip($control_points.iter()) {
            *weights *= (1.0 / control_point[0]);
        }
        let mut triangles = Vec::new();
        let signed_triangle_areas: Vec<f32> = (0..4)
            .map(|i| {
                // $control_points[j].signum()
                let points: Vec<_> = (0..4)
                    .filter(|j| i != *j)
                    .map(|j| $control_points[j])
                    .collect();
                points[0]
                    .regressive_product(points[1])
                    .regressive_product(points[2])
            })
            .collect();
        let triangle_area_sum = signed_triangle_areas[0].abs()
            + signed_triangle_areas[1].abs()
            + signed_triangle_areas[2].abs()
            + signed_triangle_areas[3].abs();
        let mut enclosing_triangle = None;
        for (triangle_index, signed_triangle_area) in signed_triangle_areas.iter().enumerate() {
            let equilibrium: f32 = 0.5 * triangle_area_sum;
            if (equilibrium - signed_triangle_area.abs()).abs() <= ERROR_MARGIN {
                enclosing_triangle = if enclosing_triangle.is_none() {
                    Some(triangle_index)
                } else {
                    None
                };
            }
        }
        if let Some(enclosing_triangle) = enclosing_triangle {
            emit_cubic_curve_triangle!(
                triangles,
                signed_triangle_areas,
                $control_points,
                $weights,
                $v,
                $w,
                $emit_vertex,
                enclosing_triangle
            );
        } else {
            let mut opposite_triangle = 0;
            for j in 1..4 {
                let side_of_a = signed_triangle_areas[j];
                let side_of_d = signed_triangle_areas[0] * if j == 2 { -1.0 } else { 1.0 };
                if side_of_a * side_of_d < 0.0 {
                    assert_eq!(opposite_triangle, 0);
                    opposite_triangle = j;
                }
            }
            assert_ne!(opposite_triangle, 0);
            emit_cubic_curve_triangle!(
                triangles,
                signed_triangle_areas,
                $control_points,
                $weights,
                $v,
                $w,
                $emit_vertex,
                0
            );
            emit_cubic_curve_triangle!(
                triangles,
                signed_triangle_areas,
                $control_points,
                $weights,
                $v,
                $w,
                $emit_vertex,
                opposite_triangle
            );
        }
        let mut additional_vertices = 0;
        for i in 1..3 {
            if enclosing_triangle != Some(i) && implicit_curve_value($weights[i]) < 0.0 {
                $fill_solid_vertices.push(point_to_vec($control_points[i]));
                additional_vertices += 1;
            }
        }
        if additional_vertices == 2 && signed_triangle_areas[0] * signed_triangle_areas[1] < 0.0 {
            let length = $fill_solid_vertices.len();
            $fill_solid_vertices.swap(length - 2, length - 1);
        }
        $cubic_vertices.append(&mut triangles);
    }};
}

macro_rules! split_curve_at {
    ($algebra:ident, $control_points:expr, $param:expr) => {{
        let p10 = $control_points[0] * (1.0 - $param) + $control_points[1] * $param;
        let p11 = $control_points[1] * (1.0 - $param) + $control_points[2] * $param;
        let p12 = $control_points[2] * (1.0 - $param) + $control_points[3] * $param;
        let p20 = p10 * (1.0 - $param) + p11 * $param;
        let p21 = p11 * (1.0 - $param) + p12 * $param;
        let p30 = p20 * (1.0 - $param) + p21 * $param;
        (
            [$control_points[0], p10, p20, p30],
            [p30, p21, p12, $control_points[3]],
        )
    }};
}

macro_rules! emit_cubic_curve {
    ($proto_hull:expr, $fill_solid_vertices:expr, $cubic_vertices:expr,
     $control_points:expr, $c:expr, $discriminant:expr, $roots:expr,
     $v:ident, $w:ident, $emit_vertex:expr) => {{
        let mut weights = weights($discriminant, &$roots);
        let mut planes = weight_planes(&$control_points, &weights);
        let gradient = implicit_curve_gradient(&planes, &weights[0]);
        normalize_implicit_curve_side(&mut planes, &mut weights, &$c, gradient);
        let mut weights = [
            ppga3d::Point::from(<[f32; 4] as Into<[f32; 4]>>::into(weights[0])),
            ppga3d::Point::from(<[f32; 4] as Into<[f32; 4]>>::into(weights[1])),
            ppga3d::Point::from(<[f32; 4] as Into<[f32; 4]>>::into(weights[2])),
            ppga3d::Point::from(<[f32; 4] as Into<[f32; 4]>>::into(weights[3])),
        ];
        if let Some(param) = find_double_point_issue($discriminant, &$roots) {
            let (control_points_a, control_points_b) =
                split_curve_at!(ppga2d, &$control_points, param);
            let (mut weights_a, mut weights_b) = split_curve_at!(ppga3d, &weights, param);
            triangulate_cubic_curve_quadrilateral!(
                $fill_solid_vertices,
                $cubic_vertices,
                &control_points_a,
                weights_a,
                $v,
                $w,
                $emit_vertex
            );
            $fill_solid_vertices.push(point_to_vec(control_points_b[0]));
            for weights in &mut weights_b {
                weights[0] *= -1.0;
                weights[1] *= -1.0;
            }
            triangulate_cubic_curve_quadrilateral!(
                $fill_solid_vertices,
                $cubic_vertices,
                &control_points_b,
                weights_b,
                $v,
                $w,
                $emit_vertex
            );
        } else {
            triangulate_cubic_curve_quadrilateral!(
                $fill_solid_vertices,
                $cubic_vertices,
                $control_points,
                weights,
                $v,
                $w,
                $emit_vertex
            );
        }
        $proto_hull.push(point_to_vec($control_points[1]).into());
        $proto_hull.push(point_to_vec($control_points[2]).into());
        $proto_hull.push(point_to_vec($control_points[3]).into());
        $fill_solid_vertices.push(point_to_vec($control_points[3]));
    }};
}

#[derive(Default)]
pub struct FillBuilder {
    pub solid_indices: Vec<u16>,
    pub solid_vertices: Vec<Vertex0>,
    pub integral_quadratic_vertices: Vec<Vertex2f>,
    pub integral_cubic_vertices: Vec<Vertex3f>,
    pub rational_quadratic_vertices: Vec<Vertex3f>,
    pub rational_cubic_vertices: Vec<Vertex4f>,
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
        let mut line_segment_iter = path.line_segments.iter();
        let mut integral_quadratic_curve_segment_iter =
            path.integral_quadratic_curve_segments.iter();
        let mut integral_cubic_curve_segment_iter = path.integral_cubic_curve_segments.iter();
        let mut rational_quadratic_curve_segment_iter =
            path.rational_quadratic_curve_segments.iter();
        let mut rational_cubic_curve_segment_iter = path.rational_cubic_curve_segments.iter();
        for segment_type in &path.segment_types {
            match segment_type {
                SegmentType::Line => {
                    let segment = line_segment_iter.next().unwrap();
                    proto_hull.push(segment.control_points[0]);
                    path_solid_vertices.push(segment.control_points[0].unwrap());
                }
                SegmentType::IntegralQuadraticCurve => {
                    let segment = integral_quadratic_curve_segment_iter.next().unwrap();
                    self.integral_quadratic_vertices
                        .push(Vertex2f(segment.control_points[1].unwrap(), [1.0, 1.0]));
                    self.integral_quadratic_vertices
                        .push(Vertex2f(segment.control_points[0].unwrap(), [0.5, 0.0]));
                    self.integral_quadratic_vertices
                        .push(Vertex2f(*path_solid_vertices.last().unwrap(), [0.0, 0.0]));
                    proto_hull.push(segment.control_points[0]);
                    proto_hull.push(segment.control_points[1]);
                    path_solid_vertices.push(segment.control_points[1].unwrap());
                }
                SegmentType::IntegralCubicCurve => {
                    let segment = integral_cubic_curve_segment_iter.next().unwrap();
                    let control_points = [
                        vec_to_point(*path_solid_vertices.last().unwrap()),
                        vec_to_point(segment.control_points[0].unwrap()),
                        vec_to_point(segment.control_points[1].unwrap()),
                        vec_to_point(segment.control_points[2].unwrap()),
                    ];
                    let power_basis = rational_cubic_control_points_to_power_basis(&control_points);
                    let ippc = inflection_point_polynomial_coefficients(&power_basis, true);
                    let (discriminant, roots) = integral_inflection_points(&ippc, true);
                    emit_cubic_curve!(
                        proto_hull,
                        path_solid_vertices,
                        self.integral_cubic_vertices,
                        control_points,
                        power_basis,
                        discriminant,
                        roots,
                        v,
                        w,
                        Vertex3f(v, [w[0], w[1], w[2]])
                    );
                }
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
                SegmentType::RationalCubicCurve => {
                    let segment = rational_cubic_curve_segment_iter.next().unwrap();
                    let weights = segment.weights.unwrap();
                    let control_points = [
                        weighted_vec_to_point(weights[0], *path_solid_vertices.last().unwrap()),
                        weighted_vec_to_point(weights[1], segment.control_points[0].unwrap()),
                        weighted_vec_to_point(weights[2], segment.control_points[1].unwrap()),
                        weighted_vec_to_point(weights[3], segment.control_points[2].unwrap()),
                    ];
                    let power_basis = rational_cubic_control_points_to_power_basis(&control_points);
                    let ippc = inflection_point_polynomial_coefficients(&power_basis, false);
                    let (discriminant, roots) = rational_inflection_points(&ippc, true);
                    emit_cubic_curve!(
                        proto_hull,
                        path_solid_vertices,
                        self.rational_cubic_vertices,
                        control_points,
                        power_basis,
                        discriminant,
                        roots,
                        v,
                        w,
                        Vertex4f(v, w.into())
                    );
                }
            }
        }
        let start_index = self.solid_vertices.len();
        self.solid_vertices
            .append(&mut triangle_fan_to_triangles(path_solid_vertices));
        let mut indices: Vec<u16> =
            (start_index as u16..(self.solid_vertices.len()) as u16).collect();
        self.solid_indices.append(&mut indices);
        Ok(())
    }
}
