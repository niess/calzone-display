use bevy::prelude::*;
use bevy::render::mesh::{
    Extrudable, Indices, PerimeterSegment, PrimitiveTopology, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
use crate::view_transform;
use super::data::{BoxInfo, MeshInfo, OrbInfo, SolidInfo, SphereInfo, TubsInfo};
use super::units::Meters;

pub(crate) trait IntoMesh {
    fn into_mesh(self) -> Mesh;
}

impl IntoMesh for SolidInfo {
    fn into_mesh(self) -> Mesh {
        match self {
            SolidInfo::Box(solid) => solid.into_mesh(),
            SolidInfo::Mesh(solid) => solid.into_mesh(),
            SolidInfo::Orb(solid) => solid.into_mesh(),
            SolidInfo::Sphere(solid) => solid.into_mesh(),
            SolidInfo::Tubs(solid) => solid.into_mesh(),
        }
    }
}

impl IntoMesh for BoxInfo  {
    fn into_mesh(self) -> Mesh {
        let size: Vec3 = std::array::from_fn(|i| self.size[i].meters()).into();
        let mut mesh: Mesh = Cuboid::from_size(size).into();
        apply_any_displacement(&mut mesh, &self.displacement);
        mesh.transform_by(view_transform());
        mesh
    }
}

fn apply_any_displacement(mesh: &mut Mesh, displacement: &[f64; 3]) {
    if displacement.iter().map(|x| x.abs()).sum::<f64>() > 0.0 {
        let displacement: [f32; 3] = std::array::from_fn(|i| displacement[i].meters());
        mesh.translate_by(displacement.into());
    }
}

impl IntoMesh for OrbInfo {
    fn into_mesh(self) -> Mesh {
        let mut mesh = Sphere::new(self.radius.meters())
            .mesh()
            .ico(7)
            .unwrap_or_else(|err| panic!("{}", err));
        apply_any_displacement(&mut mesh, &self.displacement);
        mesh.transform_by(view_transform());
        mesh
    }
}

impl IntoMesh for SphereInfo {
    fn into_mesh(self) -> Mesh {
        let mut mesh = if
            (self.delta_phi < std::f64::consts::TAU - f32::EPSILON as f64) ||
            (self.delta_theta < std::f64::consts::PI - f32::EPSILON as f64) {
            UvSphereBuilder::new(self)
                .build()
        } else {
            let mut mesh = Sphere::new(self.outer_radius.meters())
                .mesh()
                .ico(7)
                .unwrap_or_else(|err| panic!("{}", err));
            let mut inner = Sphere::new(self.inner_radius.meters())
                .mesh()
                .ico(7)
                .unwrap_or_else(|err| panic!("{}", err))
                .with_inverted_winding()
                .unwrap_or_else(|err| panic!("{}", err));
            let VertexAttributeValues::Float32x3(normals) =
                inner.attribute_mut(Mesh::ATTRIBUTE_NORMAL).unwrap() else { unreachable!() };
            for n in normals.iter_mut() {
                *n = [-n[0], -n[1], -n[2]];
            }
            mesh.merge(&inner)
                .unwrap_or_else(|err| panic!("{}", err));
            mesh
        };
        mesh.transform_by(view_transform());
        mesh
    }
}

impl IntoMesh for MeshInfo {
    fn into_mesh(self) -> Mesh {
        let n = self.0.len() / 3;
        let mut vertices = Vec::with_capacity(n); // Vertices are duplicated in order to properly
        let mut normals = Vec::with_capacity(n);  // apply faces normals.
        let mut indices = Vec::with_capacity(n);

        for (i, facet) in self.0.chunks_exact(9).enumerate() {
            let v: [[f32; 3]; 3] = std::array::from_fn(|j| {
                let v = &facet[(3 * j)..(3 * (j + 1))];
                let r = std::array::from_fn(|k| v[k].meters());
                let r = view_transform() * Vec3::from(r);
                r.into()
            });

            let normal: [f32; 3] = MeshData::compute_normal(&v).into();

            for j in 0..3 {
                vertices.push(v[j]);
                indices.push((3 * i + j) as u32);
                normals.push(normal);
            }
        }

        MeshData { vertices, normals, indices }.into_mesh()
    }
}

impl IntoMesh for TubsInfo  {
    fn into_mesh(self) -> Mesh {
        const RESOLUTION: u32 = 256;
        let mut mesh = if self.inner_radius == 0.0 {
            if self.delta_phi >= std::f64::consts::TAU {
                let mut mesh = Cylinder::new(
                    self.outer_radius.meters(),
                    self.length.meters(),
                )
                    .mesh()
                    .resolution(RESOLUTION)
                    .build();
                apply_any_displacement(&mut mesh, &self.displacement);
                let quat = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                mesh.rotate_by(quat);
                mesh
            } else {
                let sector = CircularSector::new(
                    self.outer_radius.meters(),
                    (0.5 * self.delta_phi) as f32,
                );
                let mut mesh = Extrusion::new(sector, self.length.meters())
                    .mesh()
                    .build();
                let angle = (
                    self.start_phi + 0.5 * self.delta_phi - std::f64::consts::FRAC_PI_2
                ) as f32;
                if angle.abs() > f32::EPSILON {
                    let quat = Quat::from_rotation_z(angle);
                    mesh.rotate_by(quat);
                }
                mesh
            }
        } else {
            if self.delta_phi >= std::f64::consts::TAU {
                let annulus = Annulus::new(
                    self.inner_radius.meters(),
                    self.outer_radius.meters(),
                );
                Extrusion::new(annulus, self.length.meters())
                    .mesh()
                    .resolution(RESOLUTION)
                    .build()
            } else {
                let sector = AnnulusSector {
                    inner_radius: self.inner_radius.meters(),
                    outer_radius: self.outer_radius.meters(),
                    half_angle: (0.5 * self.delta_phi) as f32,
                };
                let mut mesh = Extrusion::new(sector, self.length.meters())
                    .mesh()
                    .build();
                let angle = (
                    self.start_phi + 0.5 * self.delta_phi - std::f64::consts::FRAC_PI_2
                ) as f32;
                if angle.abs() > f32::EPSILON {
                    let quat = Quat::from_rotation_z(angle);
                    mesh.rotate_by(quat);
                }
                mesh
            }
        };
        mesh.transform_by(view_transform());
        mesh
    }
}

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn compute_normal<T>(vertices: &[T; 3]) -> Vec3
    where
        T: Copy,
        Vec3: From<T>,
    {
        let v0 = Vec3::from(vertices[0]);
        let v1 = Vec3::from(vertices[1]);
        let v2 = Vec3::from(vertices[2]);
        (v1 - v0).cross(v2 - v0).normalize()
    }
}

impl IntoMesh for MeshData {
    fn into_mesh(self) -> Mesh {
        let vertices = VertexAttributeValues::Float32x3(self.vertices);
        let normals = VertexAttributeValues::Float32x3(self.normals);
        let indices = Indices::U32(self.indices);

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(indices)
    }
}

#[derive(Clone, Copy)]
struct AnnulusSector {
    inner_radius: f32,
    outer_radius: f32,
    half_angle: f32,
}

struct AnnulusSectorBuilder {
    sector: AnnulusSector,
    resolution: u32,
}

struct RawMesh {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl Meshable for AnnulusSector {
    type Output = AnnulusSectorBuilder;

    fn mesh(&self) -> Self::Output {
        let resolution = (((self.half_angle / std::f32::consts::PI) * 256.0) as u32).max(16);
        AnnulusSectorBuilder {
            sector: *self,
            resolution,
        }
    }
}

impl Primitive2d for AnnulusSector {}

impl MeshBuilder for AnnulusSectorBuilder {
    fn build(&self) -> Mesh {
        // Adapted from Bevy/AnnulusMeshBuilder.
        let inner_radius = self.sector.inner_radius;
        let outer_radius = self.sector.outer_radius;
        let half_angle = self.sector.half_angle;
        let resolution = self.resolution as usize;

        let num_vertices = 2 * resolution;
        let mut indices = Vec::with_capacity(6 * (resolution - 1));
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut uvs = Vec::with_capacity(num_vertices);
        let normals = vec![[0.0, 0.0, 1.0]; num_vertices];

        // Each iteration places a pair of vertices at a fixed angle from the center of the
        // annulus.
        let start_angle = -half_angle + std::f32::consts::FRAC_PI_2;
        let step = 2.0 * half_angle / (resolution - 1) as f32;
        for i in 0..resolution {
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            let inner_pos = [cos * inner_radius, sin * inner_radius, 0.];
            let outer_pos = [cos * outer_radius, sin * outer_radius, 0.];
            vertices.push(inner_pos);
            vertices.push(outer_pos);

            // The first UV direction is radial and the second is angular; i.e., a single UV
            // rectangle is stretched around the annulus, with its top and bottom meeting as the
            // circle closes. Lines of constant U map to circles, and lines of constant V map to
            // radial line segments.
            let inner_uv = [0., i as f32 / (resolution - 1) as f32];
            let outer_uv = [1., i as f32 / (resolution - 1) as f32];
            uvs.push(inner_uv);
            uvs.push(outer_uv);
        }

        // Adjacent pairs of vertices form two triangles with each other; here, we are just making
        // sure that they both have the right orientation, which is the CCW order of
        // `inner_vertex` -> `outer_vertex` -> `next_outer` -> `next_inner`
        for i in 0..((resolution - 1) as u32) {
            let inner_vertex = 2 * i;
            let outer_vertex = 2 * i + 1;
            let next_inner = inner_vertex + 2;
            let next_outer = outer_vertex + 2;
            indices.extend_from_slice(&[inner_vertex, outer_vertex, next_outer]);
            indices.extend_from_slice(&[next_outer, next_inner, inner_vertex]);
        }

        RawMesh { indices, vertices, normals, uvs }
            .into()
    }
}

impl Extrudable for AnnulusSectorBuilder {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let vert_count = 2 * self.resolution;
        let (s, c) = self.sector.half_angle.sin_cos();
        vec![
            PerimeterSegment::Flat {
                indices: vec![0, 1],
            },
            PerimeterSegment::Smooth {
                first_normal: Vec2 { x: c, y: -s },
                last_normal: Vec2 { x: c, y: s },
                indices: (0..vert_count).step_by(2).rev().collect(),
            },
            PerimeterSegment::Smooth {
                first_normal: Vec2 { x: -c, y: -s },
                last_normal: Vec2 { x: -c, y: s },
                indices: (1..vert_count).step_by(2).collect(),
            },
            PerimeterSegment::Flat {
                indices: vec![vert_count - 1, vert_count - 2],
            },
        ]
    }
}

struct UvSphereBuilder {
    sphere: SphereInfo,
    sectors: usize,
    stacks: usize,
}

enum CapKind {
    Lower,
    Upper,
}

enum SideKind {
    Left,
    Right,
}

enum ShellKind {
    Inner,
    Outer,
}

impl UvSphereBuilder {
    fn new(sphere: SphereInfo) -> Self {
        let sectors = (((sphere.delta_phi / std::f64::consts::PI) * 72.0) as usize).max(4);
        let stacks = (((sphere.delta_theta / std::f64::consts::PI) * 32.0) as usize).max(2);
        Self { sphere, sectors, stacks }
    }

    fn build_cap(&self, theta_lim: f32, kind: CapKind) -> Mesh {
        let phi_step = self.sphere.delta_phi as f32 / self.sectors as f32;
        let sgn = match kind {
            CapKind::Lower => 1.0,
            CapKind::Upper => -1.0,
        };

        let raw = if self.sphere.inner_radius <= f32::EPSILON as f64 {
            let radius = self.sphere.outer_radius.meters();

            let n_vertices = self.sectors + 2;
            let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
            let mut indices: Vec<u32> = Vec::with_capacity(3 * self.sectors);

            let (st, ct) = theta_lim.sin_cos();
            let rho = radius * st;
            let z = radius * ct;
            vertices.push([0.0, 0.0, 0.0]);
            let phi0 = self.sphere.start_phi as f32 + 0.5 * phi_step;
            let (sp, cp) = phi0.sin_cos();
            normals.push([sgn * ct * cp, sgn * ct * sp, -sgn * st]);
            uvs.push([0.0, 1.0]);
            for j in 0..self.sectors + 1 {
                let phi = self.sphere.start_phi as f32 + (j as f32) * phi_step;
                let (sp, cp) = phi.sin_cos();
                let x = rho * cp;
                let y = rho * sp;

                vertices.push([x, y, z]);
                normals.push([sgn * ct * cp, sgn * ct * sp, -sgn * st]);
                uvs.push([(j as f32) / self.sectors as f32, 0.0]);
            }
            for j in 0..self.sectors as u32 {
                match kind {
                    CapKind::Lower => {
                        indices.push(0);
                        indices.push(j + 2);
                        indices.push(j + 1);
                    },
                    CapKind::Upper => {
                        indices.push(0);
                        indices.push(j + 1);
                        indices.push(j + 2);
                    },
                }
            }
            RawMesh { vertices, normals, uvs, indices }
        } else {
            let inner_radius = self.sphere.inner_radius.meters();
            let outer_radius = self.sphere.outer_radius.meters();

            let n_vertices = 2 * (self.sectors + 1);
            let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
            let mut indices: Vec<u32> = Vec::with_capacity(6 * self.sectors);

            let (st, ct) = theta_lim.sin_cos();
            let inner_rho = inner_radius * st;
            let outer_rho = outer_radius * st;
            let inner_z = inner_radius * ct;
            let outer_z = outer_radius * ct;
            let vmin = 1.0 - inner_radius / outer_radius;
            for j in 0..self.sectors + 1 {
                let phi = self.sphere.start_phi as f32 + (j as f32) * phi_step;
                let (sp, cp) = phi.sin_cos();

                vertices.extend([
                    [inner_rho * cp, inner_rho * sp, inner_z],
                    [outer_rho * cp, outer_rho * sp, outer_z],
                ]);
                normals.extend([
                    [sgn * ct * cp, sgn * ct * sp, -sgn * st],
                    [sgn * ct * cp, sgn * ct * sp, -sgn * st],
                ]);
                uvs.extend([
                    [(j as f32) / self.sectors as f32, 0.0],
                    [(j as f32) / self.sectors as f32, vmin],
                ]);
            }
            for j in 0..self.sectors as u32 {
                match kind {
                    CapKind::Lower => {
                        indices.push(2 * j);
                        indices.push(2 * j + 2);
                        indices.push(2 * j + 1);
                        indices.push(2 * j + 2);
                        indices.push(2 * j + 3);
                        indices.push(2 * j + 1);
                    },
                    CapKind::Upper => {
                        indices.push(2 * j);
                        indices.push(2 * j + 1);
                        indices.push(2 * j + 2);
                        indices.push(2 * j + 2);
                        indices.push(2 * j + 1);
                        indices.push(2 * j + 3);
                    },
                }
            }
            RawMesh { vertices, normals, uvs, indices }
        };
        raw.into()
    }

    fn build_side(&self, phi_lim: f32, kind: SideKind) -> Mesh {
        let theta_step = self.sphere.delta_theta as f32 / self.stacks as f32;
        let (sp, cp) = phi_lim.sin_cos();
        let normal = match kind {
            SideKind::Left => [sp, -cp, 0.0],
            SideKind::Right =>[-sp, cp, 0.0],
        };

        let raw = if self.sphere.inner_radius <= f32::EPSILON as f64 {
            let radius = self.sphere.outer_radius.meters();

            let n_vertices = self.stacks + 2;
            let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
            let mut indices: Vec<u32> = Vec::with_capacity(3 * self.stacks);

            vertices.push([0.0, 0.0, 0.0]);
            normals.push(normal);
            uvs.push([0.0, 1.0]);
            for i in 0..self.stacks + 1 {
                let theta = self.sphere.start_theta as f32 + (i as f32) * theta_step;
                let (st, ct) = theta.sin_cos();
                let rho = radius * st;
                let x = rho * cp;
                let y = rho * sp;
                let z = radius * ct;

                vertices.push([x, y, z]);
                normals.push(normal);
                uvs.push([(i as f32) / self.stacks as f32, 0.0]);
            }
            for i in 0..self.stacks as u32 {
                match kind {
                    SideKind::Left => {
                        indices.push(0);
                        indices.push(i + 2);
                        indices.push(i + 1);
                    },
                    SideKind::Right => {
                        indices.push(0);
                        indices.push(i + 1);
                        indices.push(i + 2);
                    },
                }
            }
            RawMesh { vertices, normals, uvs, indices }
        } else {
            let inner_radius = self.sphere.inner_radius.meters();
            let outer_radius = self.sphere.outer_radius.meters();

            let n_vertices = 2 * (self.stacks + 1);
            let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
            let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
            let mut indices: Vec<u32> = Vec::with_capacity(6 * self.stacks);

            let vmin = 1.0 - inner_radius / outer_radius;
            for i in 0..self.stacks + 1 {
                let theta = self.sphere.start_theta as f32 + (i as f32) * theta_step;
                let (st, ct) = theta.sin_cos();
                let inner_rho = inner_radius * st;
                let outer_rho = outer_radius * st;

                vertices.extend([
                    [inner_rho * cp, inner_rho * sp, inner_radius * ct],
                    [outer_rho * cp, outer_rho * sp, outer_radius * ct],
                ]);
                normals.extend([normal, normal]);
                uvs.extend([
                    [(i as f32) / self.stacks as f32, 0.0],
                    [(i as f32) / self.stacks as f32, vmin],
                ]);
            }
            for i in 0..self.stacks as u32 {
                match kind {
                    SideKind::Left => {
                        indices.push(2 * i);
                        indices.push(2 * i + 2);
                        indices.push(2 * i + 1);
                        indices.push(2 * i + 2);
                        indices.push(2 * i + 3);
                        indices.push(2 * i + 1);
                    },
                    SideKind::Right => {
                        indices.push(2 * i);
                        indices.push(2 * i + 1);
                        indices.push(2 * i + 2);
                        indices.push(2 * i + 2);
                        indices.push(2 * i + 1);
                        indices.push(2 * i + 3);
                    },
                }
            }
            RawMesh { vertices, normals, uvs, indices }
        };
        raw.into()
    }

    fn build_shell(&self, kind: ShellKind) -> Mesh {
        let (radius, sgn) = match kind {
            ShellKind::Inner => (
                self.sphere.inner_radius.meters(),
                -1.0,
            ),
            ShellKind::Outer => (
                self.sphere.outer_radius.meters(),
                1.0,
            ),
        };
        let phi_step = self.sphere.delta_phi as f32 / self.sectors as f32;
        let theta_step = self.sphere.delta_theta as f32 / self.stacks as f32;

        let n_vertices = (self.stacks + 1) * (self.sectors + 1);
        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(6 * self.stacks * self.sectors);

        for i in 0..self.stacks + 1 {
            let theta = self.sphere.start_theta as f32 + (i as f32) * theta_step;
            let (st, ct) = theta.sin_cos();
            let rho = radius * st;
            let z = radius * ct;

            for j in 0..self.sectors + 1 {
                let phi = self.sphere.start_phi as f32 + (j as f32) * phi_step;
                let (sp, cp) = phi.sin_cos();
                let x = rho * cp;
                let y = rho * sp;

                vertices.push([x, y, z]);
                normals.push([sgn * st * cp, sgn * st * sp, sgn * ct]);
                uvs.push([(j as f32) / self.sectors as f32, (i as f32) / self.stacks as f32]);
            }
        }

        // indices
        //  k1--k1+1
        //  |  / |
        //  | /  |
        //  k2--k2+1
        for i in 0..self.stacks as u32 {
            let mut k1 = i *(self.sectors as u32 + 1);
            let mut k2 = k1 + self.sectors as u32 + 1;
            for _j in 0..self.sectors as u32 {
                match kind {
                    ShellKind::Inner => {
                        indices.push(k1);
                        indices.push(k1 + 1);
                        indices.push(k2);
                        indices.push(k1 + 1);
                        indices.push(k2 + 1);
                        indices.push(k2);
                    },
                    ShellKind::Outer => {
                        indices.push(k1);
                        indices.push(k2);
                        indices.push(k1 + 1);
                        indices.push(k1 + 1);
                        indices.push(k2);
                        indices.push(k2 + 1);
                    },
                }
                k1 += 1;
                k2 += 1;
            }
        }

        RawMesh { indices, vertices, normals, uvs }
            .into()
    }
}

impl MeshBuilder for UvSphereBuilder {
    fn build(&self) -> Mesh {
        let mut mesh = self.build_shell(ShellKind::Outer);
        if self.sphere.inner_radius > 0.0 {
            let inner_shell = self.build_shell(ShellKind::Inner);
            mesh.merge(&inner_shell).unwrap();
        }
        if self.sphere.delta_theta < std::f64::consts::PI - f32::EPSILON as f64 {
            let (theta_min, theta_max) = {
                let t0 = self.sphere.start_theta as f32;
                let t1 = (self.sphere.start_theta + self.sphere.delta_theta) as f32;
                if t0 <= t1 {
                    (t0, t1)
                } else {
                    (t1, t0)
                }
            };
            if theta_min > 0.0 {
                let upper_cap = self.build_cap(theta_min, CapKind::Upper);
                mesh.merge(&upper_cap).unwrap();
            }
            if theta_max < std::f32::consts::PI {
                let lower_cap = self.build_cap(theta_max, CapKind::Lower);
                mesh.merge(&lower_cap).unwrap();
            }
        }
        if self.sphere.delta_phi < std::f64::consts::TAU - f32::EPSILON as f64 {
            let phi0 = self.sphere.start_phi as f32;
            let phi1 = (self.sphere.start_phi + self.sphere.delta_phi) as f32;
            let left_side = self.build_side(phi0, SideKind::Left);
            mesh.merge(&left_side).unwrap();
            let right_side = self.build_side(phi1, SideKind::Right);
            mesh.merge(&right_side).unwrap();
        }
        mesh
    }
}

impl From<RawMesh> for Mesh {
    fn from(value: RawMesh) -> Self {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
            .with_inserted_indices(Indices::U32(value.indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, value.vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, value.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs)
    }
}
