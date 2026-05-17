use alloc::vec::Vec;

use scenix_math::{Vec2, Vec3};

use crate::{EPSILON, Geometry, Shape, clamp, sin_cos};

/// Creates segmented box geometry centered at the origin.
pub fn box_geometry(
    width: f32,
    height: f32,
    depth: f32,
    width_segments: u32,
    height_segments: u32,
    depth_segments: u32,
) -> Geometry {
    if width <= 0.0 || height <= 0.0 || depth <= 0.0 {
        return Geometry::new();
    }

    let mut geometry = Geometry::new();
    let ws = width_segments.max(1);
    let hs = height_segments.max(1);
    let ds = depth_segments.max(1);
    let hw = width * 0.5;
    let hh = height * 0.5;
    let hd = depth * 0.5;

    add_grid_face(
        &mut geometry,
        Vec3::new(hw, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -depth),
        Vec3::new(0.0, height, 0.0),
        Vec3::X,
        ds,
        hs,
    );
    add_grid_face(
        &mut geometry,
        Vec3::new(-hw, 0.0, 0.0),
        Vec3::new(0.0, 0.0, depth),
        Vec3::new(0.0, height, 0.0),
        -Vec3::X,
        ds,
        hs,
    );
    add_grid_face(
        &mut geometry,
        Vec3::new(0.0, hh, 0.0),
        Vec3::new(width, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -depth),
        Vec3::Y,
        ws,
        ds,
    );
    add_grid_face(
        &mut geometry,
        Vec3::new(0.0, -hh, 0.0),
        Vec3::new(width, 0.0, 0.0),
        Vec3::new(0.0, 0.0, depth),
        -Vec3::Y,
        ws,
        ds,
    );
    add_grid_face(
        &mut geometry,
        Vec3::new(0.0, 0.0, hd),
        Vec3::new(width, 0.0, 0.0),
        Vec3::new(0.0, height, 0.0),
        Vec3::Z,
        ws,
        hs,
    );
    add_grid_face(
        &mut geometry,
        Vec3::new(0.0, 0.0, -hd),
        Vec3::new(-width, 0.0, 0.0),
        Vec3::new(0.0, height, 0.0),
        -Vec3::Z,
        ws,
        hs,
    );

    geometry
}

/// Creates segmented plane geometry in the XY plane.
pub fn plane_geometry(
    width: f32,
    height: f32,
    width_segments: u32,
    height_segments: u32,
) -> Geometry {
    if width <= 0.0 || height <= 0.0 {
        return Geometry::new();
    }
    let mut geometry = Geometry::new();
    add_grid_face(
        &mut geometry,
        Vec3::ZERO,
        Vec3::new(width, 0.0, 0.0),
        Vec3::new(0.0, height, 0.0),
        Vec3::Z,
        width_segments.max(1),
        height_segments.max(1),
    );
    geometry
}

/// Creates UV sphere geometry centered at the origin.
pub fn sphere_geometry(radius: f32, width_segments: u32, height_segments: u32) -> Geometry {
    if radius <= 0.0 {
        return Geometry::new();
    }
    let width_segments = width_segments.max(3);
    let height_segments = height_segments.max(2);
    let mut geometry = Geometry::new();
    let columns = width_segments + 1;
    let rows = height_segments + 1;
    geometry.positions.reserve((columns * rows) as usize);
    geometry.normals.reserve((columns * rows) as usize);
    geometry.uvs.reserve((columns * rows) as usize);
    geometry
        .indices
        .reserve((width_segments * height_segments * 6) as usize);

    for y in 0..=height_segments {
        let v = y as f32 / height_segments as f32;
        let phi = v * core::f32::consts::PI;
        let (sin_phi, cos_phi) = sin_cos(phi);
        for x in 0..=width_segments {
            let u = x as f32 / width_segments as f32;
            let theta = u * core::f32::consts::TAU;
            let (sin_theta, cos_theta) = sin_cos(theta);
            let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta).normalize();
            geometry.positions.push(normal * radius);
            geometry.normals.push(normal);
            geometry.uvs.push(Vec2::new(u, 1.0 - v));
        }
    }

    add_grid_indices(&mut geometry, width_segments, height_segments, true);
    geometry
}

/// Creates cylinder or truncated-cone geometry centered at the origin.
pub fn cylinder_geometry(
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    radial_segments: u32,
    height_segments: u32,
    open_ended: bool,
) -> Geometry {
    if height <= 0.0 || top_radius < 0.0 || bottom_radius < 0.0 || top_radius + bottom_radius <= 0.0
    {
        return Geometry::new();
    }

    let radial_segments = radial_segments.max(3);
    let height_segments = height_segments.max(1);
    let mut geometry = Geometry::new();
    let columns = radial_segments + 1;
    let rows = height_segments + 1;
    geometry.positions.reserve((columns * rows) as usize);
    geometry.normals.reserve((columns * rows) as usize);
    geometry.uvs.reserve((columns * rows) as usize);

    for y in 0..=height_segments {
        let v = y as f32 / height_segments as f32;
        let radius = bottom_radius + (top_radius - bottom_radius) * v;
        let py = -height * 0.5 + height * v;
        for x in 0..=radial_segments {
            let u = x as f32 / radial_segments as f32;
            let theta = u * core::f32::consts::TAU;
            let (sin_theta, cos_theta) = sin_cos(theta);
            geometry
                .positions
                .push(Vec3::new(radius * cos_theta, py, radius * sin_theta));
            let slope = (bottom_radius - top_radius) / height;
            geometry
                .normals
                .push(Vec3::new(cos_theta, slope, sin_theta).normalize());
            geometry.uvs.push(Vec2::new(u, v));
        }
    }

    add_grid_indices(&mut geometry, radial_segments, height_segments, false);

    if !open_ended {
        if top_radius > EPSILON {
            add_cap(
                &mut geometry,
                top_radius,
                height * 0.5,
                Vec3::Y,
                radial_segments,
            );
        }
        if bottom_radius > EPSILON {
            add_cap(
                &mut geometry,
                bottom_radius,
                -height * 0.5,
                -Vec3::Y,
                radial_segments,
            );
        }
    }

    geometry
}

/// Creates cone geometry centered at the origin.
#[inline]
pub fn cone_geometry(
    radius: f32,
    height: f32,
    radial_segments: u32,
    height_segments: u32,
) -> Geometry {
    cylinder_geometry(0.0, radius, height, radial_segments, height_segments, false)
}

/// Creates capsule geometry centered at the origin.
pub fn capsule_geometry(
    radius: f32,
    height: f32,
    cap_segments: u32,
    radial_segments: u32,
) -> Geometry {
    if radius <= 0.0 || height <= 0.0 {
        return Geometry::new();
    }
    let radial_segments = radial_segments.max(3);
    let cap_segments = cap_segments.max(2);
    let rings = cap_segments * 2 + 2;
    let cylinder_half = ((height - radius * 2.0).max(0.0)) * 0.5;
    let mut geometry = Geometry::new();

    for y in 0..=rings {
        let v = y as f32 / rings as f32;
        let phi = v * core::f32::consts::PI;
        let (sin_phi, cos_phi) = sin_cos(phi);
        let offset = if cos_phi >= 0.0 {
            cylinder_half
        } else {
            -cylinder_half
        };
        for x in 0..=radial_segments {
            let u = x as f32 / radial_segments as f32;
            let theta = u * core::f32::consts::TAU;
            let (sin_theta, cos_theta) = sin_cos(theta);
            let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta).normalize();
            let position = Vec3::new(
                normal.x * radius,
                normal.y * radius + offset,
                normal.z * radius,
            );
            geometry.positions.push(position);
            geometry.normals.push(normal);
            geometry.uvs.push(Vec2::new(u, 1.0 - v));
        }
    }

    add_grid_indices(&mut geometry, radial_segments, rings, true);
    geometry
}

/// Creates torus geometry around the Y axis.
pub fn torus_geometry(
    radius: f32,
    tube_radius: f32,
    radial_segments: u32,
    tubular_segments: u32,
) -> Geometry {
    if radius <= 0.0 || tube_radius <= 0.0 {
        return Geometry::new();
    }
    let radial_segments = radial_segments.max(3);
    let tubular_segments = tubular_segments.max(3);
    let mut geometry = Geometry::new();

    for j in 0..=radial_segments {
        let v = j as f32 / radial_segments as f32;
        let minor = v * core::f32::consts::TAU;
        let (sin_minor, cos_minor) = sin_cos(minor);
        for i in 0..=tubular_segments {
            let u = i as f32 / tubular_segments as f32;
            let major = u * core::f32::consts::TAU;
            let (sin_major, cos_major) = sin_cos(major);
            let center_radius = radius + tube_radius * cos_minor;
            let normal =
                Vec3::new(cos_minor * cos_major, sin_minor, cos_minor * sin_major).normalize();
            geometry.positions.push(Vec3::new(
                center_radius * cos_major,
                tube_radius * sin_minor,
                center_radius * sin_major,
            ));
            geometry.normals.push(normal);
            geometry.uvs.push(Vec2::new(u, v));
        }
    }

    add_grid_indices(&mut geometry, tubular_segments, radial_segments, false);
    geometry
}

/// Creates torus-knot geometry.
pub fn torus_knot_geometry(
    radius: f32,
    tube_radius: f32,
    tubular_segments: u32,
    radial_segments: u32,
    p: u32,
    q: u32,
) -> Geometry {
    if radius <= 0.0 || tube_radius <= 0.0 || p == 0 || q == 0 {
        return Geometry::new();
    }
    let tubular_segments = tubular_segments.max(8);
    let radial_segments = radial_segments.max(3);
    let mut centers = Vec::with_capacity((tubular_segments + 1) as usize);
    for i in 0..=tubular_segments {
        let u = i as f32 / tubular_segments as f32 * core::f32::consts::TAU * p as f32;
        centers.push(torus_knot_point(radius, p, q, u));
    }

    let mut geometry = Geometry::new();
    for i in 0..=tubular_segments {
        let prev = centers[if i == 0 {
            tubular_segments as usize - 1
        } else {
            i as usize - 1
        }];
        let next = centers[if i == tubular_segments {
            1
        } else {
            i as usize + 1
        }];
        let tangent = (next - prev).normalize();
        let mut normal = tangent.cross(Vec3::Y).normalize();
        if normal.length_squared() <= EPSILON {
            normal = tangent.cross(Vec3::X).normalize();
        }
        let binormal = tangent.cross(normal).normalize();
        for j in 0..=radial_segments {
            let v = j as f32 / radial_segments as f32;
            let angle = v * core::f32::consts::TAU;
            let (sin_angle, cos_angle) = sin_cos(angle);
            let tube_normal = (normal * cos_angle + binormal * sin_angle).normalize();
            geometry
                .positions
                .push(centers[i as usize] + tube_normal * tube_radius);
            geometry.normals.push(tube_normal);
            geometry
                .uvs
                .push(Vec2::new(i as f32 / tubular_segments as f32, v));
        }
    }

    add_grid_indices(&mut geometry, radial_segments, tubular_segments, true);
    geometry
}

/// Creates an icosphere by subdividing an icosahedron.
pub fn icosphere_geometry(radius: f32, subdivisions: u32) -> Geometry {
    if radius <= 0.0 {
        return Geometry::new();
    }
    let t = 1.618_034_f32;
    let mut vertices = alloc::vec![
        Vec3::new(-1.0, t, 0.0),
        Vec3::new(1.0, t, 0.0),
        Vec3::new(-1.0, -t, 0.0),
        Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t),
        Vec3::new(0.0, 1.0, t),
        Vec3::new(0.0, -1.0, -t),
        Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0),
        Vec3::new(t, 0.0, 1.0),
        Vec3::new(-t, 0.0, -1.0),
        Vec3::new(-t, 0.0, 1.0),
    ];
    for vertex in &mut vertices {
        *vertex = vertex.normalize();
    }

    let mut faces: Vec<[u32; 3]> = alloc::vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    for _ in 0..subdivisions.min(6) {
        let mut next_faces = Vec::with_capacity(faces.len() * 4);
        let mut midpoint_cache = Vec::new();
        for [a, b, c] in faces {
            let ab = midpoint(&mut vertices, &mut midpoint_cache, a, b);
            let bc = midpoint(&mut vertices, &mut midpoint_cache, b, c);
            let ca = midpoint(&mut vertices, &mut midpoint_cache, c, a);
            next_faces.extend_from_slice(&[[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        faces = next_faces;
    }

    let mut geometry = Geometry::new();
    geometry.positions.reserve(vertices.len());
    geometry.normals.reserve(vertices.len());
    geometry.uvs.reserve(vertices.len());
    for normal in vertices {
        geometry.positions.push(normal * radius);
        geometry.normals.push(normal);
        let u = clamp(0.5 + normal.x * 0.5, 0.0, 1.0);
        let v = clamp(0.5 - normal.y * 0.5, 0.0, 1.0);
        geometry.uvs.push(Vec2::new(u, v));
    }
    for face in faces {
        push_oriented_triangle(&mut geometry, face[0], face[1], face[2]);
    }
    geometry
}

/// Creates a filled circle or arc in the XY plane.
pub fn circle_geometry(
    radius: f32,
    segments: u32,
    theta_start: f32,
    theta_length: f32,
) -> Geometry {
    if radius <= 0.0 || theta_length <= 0.0 {
        return Geometry::new();
    }
    let segments = segments.max(3);
    let mut geometry = Geometry::new();
    geometry.positions.reserve((segments + 2) as usize);
    geometry.positions.push(Vec3::ZERO);
    geometry.normals.push(Vec3::Z);
    geometry.uvs.push(Vec2::new(0.5, 0.5));

    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let theta = theta_start + theta_length * u;
        let (sin_theta, cos_theta) = sin_cos(theta);
        let x = radius * cos_theta;
        let y = radius * sin_theta;
        geometry.positions.push(Vec3::new(x, y, 0.0));
        geometry.normals.push(Vec3::Z);
        geometry.uvs.push(Vec2::new(
            0.5 + x / (2.0 * radius),
            0.5 + y / (2.0 * radius),
        ));
    }

    for i in 1..=segments {
        geometry.indices.extend_from_slice(&[0, i, i + 1]);
    }
    geometry
}

/// Creates ring geometry in the XY plane.
pub fn ring_geometry(
    inner_radius: f32,
    outer_radius: f32,
    theta_segments: u32,
    phi_segments: u32,
) -> Geometry {
    if inner_radius < 0.0 || outer_radius <= inner_radius {
        return Geometry::new();
    }
    let theta_segments = theta_segments.max(3);
    let phi_segments = phi_segments.max(1);
    let mut geometry = Geometry::new();

    for y in 0..=phi_segments {
        let v = y as f32 / phi_segments as f32;
        let radius = inner_radius + (outer_radius - inner_radius) * v;
        for x in 0..=theta_segments {
            let u = x as f32 / theta_segments as f32;
            let theta = u * core::f32::consts::TAU;
            let (sin_theta, cos_theta) = sin_cos(theta);
            geometry
                .positions
                .push(Vec3::new(radius * cos_theta, radius * sin_theta, 0.0));
            geometry.normals.push(Vec3::Z);
            geometry.uvs.push(Vec2::new(
                0.5 + (radius * cos_theta) / (2.0 * outer_radius),
                0.5 + (radius * sin_theta) / (2.0 * outer_radius),
            ));
        }
    }

    add_grid_indices(&mut geometry, theta_segments, phi_segments, false);
    geometry
}

/// Creates a lathe surface by rotating 2D radius/Y points around the Y axis.
pub fn lathe_geometry(points: &[Vec2], segments: u32, phi_start: f32, phi_length: f32) -> Geometry {
    if points.len() < 2 || segments < 3 || phi_length <= 0.0 {
        return Geometry::new();
    }
    let mut geometry = Geometry::new();
    let segments = segments.max(3);
    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let phi = phi_start + phi_length * u;
        let (sin_phi, cos_phi) = sin_cos(phi);
        for (j, point) in points.iter().enumerate() {
            let radius = point.x.max(0.0);
            geometry
                .positions
                .push(Vec3::new(radius * cos_phi, point.y, radius * sin_phi));
            geometry
                .uvs
                .push(Vec2::new(u, j as f32 / (points.len() - 1) as f32));
        }
    }

    let rows = points.len() as u32 - 1;
    for i in 0..segments {
        for j in 0..rows {
            let a = i * points.len() as u32 + j;
            let b = (i + 1) * points.len() as u32 + j;
            let c = a + 1;
            let d = b + 1;
            geometry.indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    geometry.compute_normals();
    geometry
}

/// Creates extruded 2D shape geometry with sharp side walls.
pub fn extrude_geometry(
    shape: &Shape,
    depth: f32,
    _bevel_thickness: f32,
    _bevel_size: f32,
    _bevel_segments: u32,
) -> Geometry {
    if shape.is_empty() || depth <= 0.0 {
        return Geometry::new();
    }

    let Some(exterior) = shape.exterior() else {
        return Geometry::new();
    };
    let triangles = triangulate_simple_polygon(exterior);
    if triangles.is_empty() {
        return Geometry::new();
    }

    let mut geometry = Geometry::new();
    let half = depth * 0.5;
    let bounds = bounds2(exterior);

    for point in exterior {
        geometry.positions.push(Vec3::new(point.x, point.y, half));
        geometry.normals.push(Vec3::Z);
        geometry.uvs.push(shape_uv(*point, bounds));
    }
    for point in exterior {
        geometry.positions.push(Vec3::new(point.x, point.y, -half));
        geometry.normals.push(-Vec3::Z);
        geometry.uvs.push(shape_uv(*point, bounds));
    }

    let n = exterior.len() as u32;
    for [a, b, c] in &triangles {
        geometry.indices.extend_from_slice(&[*a, *b, *c]);
        geometry.indices.extend_from_slice(&[c + n, b + n, a + n]);
    }

    add_contour_side_walls(&mut geometry, exterior, half, false);
    for hole in shape.holes() {
        add_contour_side_walls(&mut geometry, hole, half, true);
    }
    geometry
}

/// Creates tube geometry around a polyline path.
pub fn tube_geometry(
    path: &[Vec3],
    tubular_segments: u32,
    radius: f32,
    radial_segments: u32,
    closed: bool,
) -> Geometry {
    if path.len() < 2 || tubular_segments == 0 || radius <= 0.0 {
        return Geometry::new();
    }
    let radial_segments = radial_segments.max(3);
    let samples = sample_path(path, tubular_segments, closed);
    if samples.len() < 2 {
        return Geometry::new();
    }

    let mut geometry = Geometry::new();
    for i in 0..samples.len() {
        let prev = if i == 0 {
            if closed {
                samples[samples.len() - 2]
            } else {
                samples[i]
            }
        } else {
            samples[i - 1]
        };
        let next = if i + 1 == samples.len() {
            if closed { samples[1] } else { samples[i] }
        } else {
            samples[i + 1]
        };
        let tangent = (next - prev).normalize();
        let mut normal = tangent.cross(Vec3::Y).normalize();
        if normal.length_squared() <= EPSILON {
            normal = tangent.cross(Vec3::X).normalize();
        }
        let binormal = tangent.cross(normal).normalize();

        for j in 0..=radial_segments {
            let v = j as f32 / radial_segments as f32;
            let angle = v * core::f32::consts::TAU;
            let (sin_angle, cos_angle) = sin_cos(angle);
            let tube_normal = (normal * cos_angle + binormal * sin_angle).normalize();
            geometry.positions.push(samples[i] + tube_normal * radius);
            geometry.normals.push(tube_normal);
            geometry
                .uvs
                .push(Vec2::new(i as f32 / (samples.len() - 1) as f32, v));
        }
    }

    add_grid_indices(
        &mut geometry,
        radial_segments,
        samples.len().saturating_sub(1) as u32,
        true,
    );
    flatten_by_face(&mut geometry);
    geometry
}

/// Creates triangulated 2D shape geometry in the XY plane.
pub fn shape_geometry(shape: &Shape) -> Geometry {
    if shape.is_empty() {
        return Geometry::new();
    }
    let Some(exterior) = shape.exterior() else {
        return Geometry::new();
    };
    let triangles = triangulate_simple_polygon(exterior);
    if triangles.is_empty() {
        return Geometry::new();
    }
    let bounds = bounds2(exterior);
    let mut geometry = Geometry::new();
    geometry.positions.reserve(exterior.len());
    geometry.normals.reserve(exterior.len());
    geometry.uvs.reserve(exterior.len());
    for point in exterior {
        geometry.positions.push(Vec3::new(point.x, point.y, 0.0));
        geometry.normals.push(Vec3::Z);
        geometry.uvs.push(shape_uv(*point, bounds));
    }
    for [a, b, c] in triangles {
        geometry.indices.extend_from_slice(&[a, b, c]);
    }
    geometry
}

fn add_grid_face(
    geometry: &mut Geometry,
    center: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
    normal: Vec3,
    u_segments: u32,
    v_segments: u32,
) {
    let base = geometry.positions.len() as u32;
    let u_segments = u_segments.max(1);
    let v_segments = v_segments.max(1);
    let vertex_count = ((u_segments + 1) * (v_segments + 1)) as usize;
    geometry.positions.reserve(vertex_count);
    geometry.normals.reserve(vertex_count);
    geometry.uvs.reserve(vertex_count);

    for y in 0..=v_segments {
        let v = y as f32 / v_segments as f32;
        for x in 0..=u_segments {
            let u = x as f32 / u_segments as f32;
            geometry
                .positions
                .push(center + u_axis * (u - 0.5) + v_axis * (v - 0.5));
            geometry.normals.push(normal);
            geometry.uvs.push(Vec2::new(u, 1.0 - v));
        }
    }

    let forward = u_axis.cross(v_axis).dot(normal) >= 0.0;
    for y in 0..v_segments {
        for x in 0..u_segments {
            let a = base + y * (u_segments + 1) + x;
            let b = a + 1;
            let c = a + u_segments + 1;
            let d = c + 1;
            if forward {
                geometry.indices.extend_from_slice(&[a, b, c, b, d, c]);
            } else {
                geometry.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }
}

fn add_grid_indices(geometry: &mut Geometry, columns: u32, rows: u32, forward: bool) {
    let stride = columns + 1;
    for y in 0..rows {
        for x in 0..columns {
            let a = y * stride + x;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            if forward {
                geometry.indices.extend_from_slice(&[a, b, c, b, d, c]);
            } else {
                geometry.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }
}

fn add_cap(geometry: &mut Geometry, radius: f32, y: f32, normal: Vec3, radial_segments: u32) {
    let center_index = geometry.positions.len() as u32;
    geometry.positions.push(Vec3::new(0.0, y, 0.0));
    geometry.normals.push(normal);
    geometry.uvs.push(Vec2::new(0.5, 0.5));
    for i in 0..=radial_segments {
        let u = i as f32 / radial_segments as f32;
        let theta = u * core::f32::consts::TAU;
        let (sin_theta, cos_theta) = sin_cos(theta);
        geometry
            .positions
            .push(Vec3::new(radius * cos_theta, y, radius * sin_theta));
        geometry.normals.push(normal);
        geometry.uvs.push(Vec2::new(
            0.5 + cos_theta * 0.5,
            if normal.y > 0.0 {
                0.5 - sin_theta * 0.5
            } else {
                0.5 + sin_theta * 0.5
            },
        ));
    }
    for i in 1..=radial_segments {
        if normal.y > 0.0 {
            geometry.indices.extend_from_slice(&[
                center_index,
                center_index + i + 1,
                center_index + i,
            ]);
        } else {
            geometry.indices.extend_from_slice(&[
                center_index,
                center_index + i,
                center_index + i + 1,
            ]);
        }
    }
}

fn torus_knot_point(radius: f32, p: u32, q: u32, u: f32) -> Vec3 {
    let (sin_u, cos_u) = sin_cos(u);
    let qu = q as f32 / p as f32 * u;
    let (sin_qu, cos_qu) = sin_cos(qu);
    let r = radius * (2.0 + cos_qu) * 0.5;
    Vec3::new(r * cos_u, radius * sin_qu * 0.5, r * sin_u)
}

fn midpoint(vertices: &mut Vec<Vec3>, cache: &mut Vec<((u32, u32), u32)>, a: u32, b: u32) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some((_, index)) = cache.iter().find(|(candidate, _)| *candidate == key) {
        return *index;
    }
    let point = (vertices[a as usize] + vertices[b as usize]).normalize();
    let index = vertices.len() as u32;
    vertices.push(point);
    cache.push((key, index));
    index
}

fn push_oriented_triangle(geometry: &mut Geometry, a: u32, b: u32, c: u32) {
    let p0 = geometry.positions[a as usize];
    let p1 = geometry.positions[b as usize];
    let p2 = geometry.positions[c as usize];
    let face = (p1 - p0).cross(p2 - p0);
    let center = (p0 + p1 + p2) / 3.0;
    if face.dot(center) >= 0.0 {
        geometry.indices.extend_from_slice(&[a, b, c]);
    } else {
        geometry.indices.extend_from_slice(&[a, c, b]);
    }
}

fn bounds2(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = points[0];
    let mut max = points[0];
    for point in &points[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    (min, max)
}

fn shape_uv(point: Vec2, bounds: (Vec2, Vec2)) -> Vec2 {
    let size = bounds.1 - bounds.0;
    Vec2::new(
        if size.x.abs() <= EPSILON {
            0.0
        } else {
            clamp((point.x - bounds.0.x) / size.x, 0.0, 1.0)
        },
        if size.y.abs() <= EPSILON {
            0.0
        } else {
            clamp((point.y - bounds.0.y) / size.y, 0.0, 1.0)
        },
    )
}

fn triangulate_simple_polygon(points: &[Vec2]) -> Vec<[u32; 3]> {
    let mut polygon = points.to_vec();
    if polygon.len() >= 2 && polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3 {
        return Vec::new();
    }
    if signed_area(&polygon) < 0.0 {
        polygon.reverse();
    }

    let mut available: Vec<usize> = (0..polygon.len()).collect();
    let mut triangles = Vec::with_capacity(polygon.len().saturating_sub(2));
    let mut guard = 0;
    while available.len() > 3 && guard < polygon.len() * polygon.len() {
        guard += 1;
        let len = available.len();
        let mut clipped = false;
        for i in 0..len {
            let prev = available[(i + len - 1) % len];
            let curr = available[i];
            let next = available[(i + 1) % len];
            if !is_convex(polygon[prev], polygon[curr], polygon[next]) {
                continue;
            }
            if available.iter().any(|candidate| {
                *candidate != prev
                    && *candidate != curr
                    && *candidate != next
                    && point_in_triangle(
                        polygon[*candidate],
                        polygon[prev],
                        polygon[curr],
                        polygon[next],
                    )
            }) {
                continue;
            }
            triangles.push([prev as u32, curr as u32, next as u32]);
            available.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }

    if available.len() == 3 {
        triangles.push([
            available[0] as u32,
            available[1] as u32,
            available[2] as u32,
        ]);
    }

    if triangles.is_empty() && polygon.len() >= 3 {
        for i in 1..polygon.len() - 1 {
            triangles.push([0, i as u32, i as u32 + 1]);
        }
    }
    triangles
}

fn signed_area(points: &[Vec2]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

fn is_convex(a: Vec2, b: Vec2, c: Vec2) -> bool {
    let ab = b - a;
    let bc = c - b;
    ab.x * bc.y - ab.y * bc.x > EPSILON
}

fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let area = edge(a, b, c).abs();
    let a0 = edge(point, a, b).abs();
    let a1 = edge(point, b, c).abs();
    let a2 = edge(point, c, a).abs();
    (a0 + a1 + a2 - area).abs() <= 1.0e-4
}

fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn add_contour_side_walls(
    geometry: &mut Geometry,
    contour: &[Vec2],
    half_depth: f32,
    inward: bool,
) {
    if contour.len() < 2 {
        return;
    }
    for i in 0..contour.len() {
        let a = contour[i];
        let b = contour[(i + 1) % contour.len()];
        if a.distance(b) <= EPSILON {
            continue;
        }
        let base = geometry.positions.len() as u32;
        let edge = b - a;
        let mut normal = Vec3::new(edge.y, -edge.x, 0.0).normalize();
        if inward {
            normal = -normal;
        }
        geometry.positions.extend_from_slice(&[
            Vec3::new(a.x, a.y, half_depth),
            Vec3::new(b.x, b.y, half_depth),
            Vec3::new(a.x, a.y, -half_depth),
            Vec3::new(b.x, b.y, -half_depth),
        ]);
        geometry.normals.extend_from_slice(&[normal; 4]);
        geometry.uvs.extend_from_slice(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ]);
        if inward {
            geometry.indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base + 1,
                base + 3,
                base + 2,
            ]);
        } else {
            geometry.indices.extend_from_slice(&[
                base,
                base + 2,
                base + 1,
                base + 1,
                base + 2,
                base + 3,
            ]);
        }
    }
}

fn sample_path(path: &[Vec3], tubular_segments: u32, closed: bool) -> Vec<Vec3> {
    if path.len() < 2 {
        return Vec::new();
    }
    if closed {
        let mut samples = Vec::with_capacity(tubular_segments as usize + 1);
        for i in 0..=tubular_segments {
            let scaled = i as usize * path.len();
            let index = (scaled / tubular_segments as usize) % path.len();
            let next = (index + 1) % path.len();
            let local = (scaled % tubular_segments as usize) as f32 / tubular_segments as f32;
            samples.push(path[index].lerp(path[next], local));
        }
        samples
    } else {
        let mut samples = Vec::with_capacity(tubular_segments as usize + 1);
        let max_segment = path.len() - 1;
        for i in 0..=tubular_segments {
            let scaled = i as usize * max_segment;
            let index = (scaled / tubular_segments as usize).min(max_segment - 1);
            let local = (scaled % tubular_segments as usize) as f32 / tubular_segments as f32;
            samples.push(path[index].lerp(path[index + 1], local));
        }
        samples
    }
}

fn flatten_by_face(geometry: &mut Geometry) {
    if geometry.indices.is_empty() {
        geometry.compute_normals();
        return;
    }
    let old_positions = geometry.positions.clone();
    let old_uvs = geometry.uvs.clone();
    let old_indices = geometry.indices.clone();
    geometry.positions.clear();
    geometry.normals.clear();
    geometry.uvs.clear();
    geometry.indices.clear();

    for triangle in old_indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        if a >= old_positions.len() || b >= old_positions.len() || c >= old_positions.len() {
            continue;
        }
        let p0 = old_positions[a];
        let p1 = old_positions[b];
        let p2 = old_positions[c];
        let normal = (p1 - p0).cross(p2 - p0).normalize();
        let base = geometry.positions.len() as u32;
        geometry.positions.extend_from_slice(&[p0, p1, p2]);
        geometry.normals.extend_from_slice(&[normal; 3]);
        if old_uvs.len() == old_positions.len() {
            geometry
                .uvs
                .extend_from_slice(&[old_uvs[a], old_uvs[b], old_uvs[c]]);
        }
        geometry
            .indices
            .extend_from_slice(&[base, base + 1, base + 2]);
    }
}
