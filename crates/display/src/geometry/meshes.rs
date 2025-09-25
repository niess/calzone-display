use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
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
        let mut mesh = if self.inner_radius == 0.0 {
            if self.delta_phi >= std::f64::consts::PI {
                let mut mesh = Cylinder::new(
                    self.outer_radius.meters(),
                    self.length.meters(),
                )
                    .mesh()
                    .resolution(256)
                    .build();
                apply_any_displacement(&mut mesh, &self.displacement);
                mesh
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        };

        for (_, values) in mesh.attributes_mut() {
            if let VertexAttributeValues::Float32x3(values) = values {
                for v in values.iter_mut() {
                    v.swap(1, 2);
                }
            }
        }

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
