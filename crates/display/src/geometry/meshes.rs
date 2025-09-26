use bevy::prelude::*;
use bevy::render::mesh::{
    Extrudable, Indices, PerimeterSegment, PrimitiveTopology, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
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
        mesh
    }
}

impl IntoMesh for SphereInfo {
    fn into_mesh(self) -> Mesh {
        unimplemented!()
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
                std::array::from_fn(|k| v[k].meters())
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
                let quat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
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
                let angle = (self.start_phi + 0.5 * self.delta_phi) as f32;
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
                    .resolution(RESOLUTION as usize)
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
                let angle = (self.start_phi + 0.5 * self.delta_phi) as f32;
                if angle.abs() > f32::EPSILON {
                    let quat = Quat::from_rotation_z(angle);
                    mesh.rotate_by(quat);
                }
                mesh
            }
        };
        apply_any_displacement(&mut mesh, &self.displacement);
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
        let mut positions = Vec::with_capacity(num_vertices);
        let mut uvs = Vec::with_capacity(num_vertices);
        let normals = vec![[0.0, 0.0, 1.0]; num_vertices];

        // Each iteration places a pair of vertices at a fixed angle from the center of the
        // annulus.
        let start_angle = -half_angle;
        let step = 2.0 * half_angle / (resolution - 1) as f32;
        for i in 0..resolution {
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            let inner_pos = [cos * inner_radius, sin * inner_radius, 0.];
            let outer_pos = [cos * outer_radius, sin * outer_radius, 0.];
            positions.push(inner_pos);
            positions.push(outer_pos);

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

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices))
    }
}

impl Extrudable for AnnulusSectorBuilder {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let vert_count = 2 * self.resolution;
        let (s, c) = self.sector.half_angle.sin_cos();
        vec![
            PerimeterSegment::Flat {
                indices: vec![1, 0],
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
                indices: vec![vert_count - 2, vert_count - 1],
            },
        ]
    }
}
