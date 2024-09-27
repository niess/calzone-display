use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use super::data::{BoxShape, CylinderShape, Shape, SphereShape, TessellatedShape};
use super::units::Meters;


impl From<Shape> for Mesh {
    fn from(value: Shape) -> Self {
        match value {
            Shape::Box(shape) => shape.into(),
            Shape::Cylinder(shape) => shape.into(),
            Shape::Envelope(_) => unreachable!(),
            Shape::Sphere(shape) => shape.into(),
            Shape::Tessellation(shape) => shape.into(),
        }
    }
}

impl From<BoxShape> for Mesh {
    fn from(value: BoxShape) -> Self {
        let size: Vec3 = std::array::from_fn(|i| value.size[i].meters()).into();
        Cuboid::from_size(size).into()
    }
}

impl From<CylinderShape> for Mesh {
    fn from(value: CylinderShape) -> Self {
        if value.thickness <= 0.0 {
            if value.section == [ 0.0, 360.0 ] {
                Cylinder::new(
                    value.radius.meters(),
                    value.length.meters(),
                )
                    .mesh()
                    .resolution(256)
                    .build()
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }
}

impl From<SphereShape> for Mesh {
    fn from(value: SphereShape) -> Self {
        if value.thickness <= 0.0 {
            if value.azimuth_section == [ 0.0, 360.0 ] && value.zenith_section == [1.0, 180.0] {
                Sphere::new(value.radius.meters())
                    .mesh()
                    .ico(7)
                    .unwrap_or_else(|err| panic!("{}", err))
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }
}

impl From<TessellatedShape> for Mesh {
    fn from(value: TessellatedShape) -> Self {
        let n = value.facets.len();
        let mut vertices = Vec::with_capacity(n); // Vertices are duplicated in order to properly
        let mut normals = Vec::with_capacity(n);  // apply faces normals.
        let mut indices = Vec::with_capacity(n);

        for (i, facet) in value.facets.chunks_exact(9).enumerate() {
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

        MeshData { vertices, normals, indices }.into()
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

impl From<MeshData> for Mesh {
    fn from(value: MeshData) -> Self {
        let vertices = VertexAttributeValues::Float32x3(value.vertices);
        let normals = VertexAttributeValues::Float32x3(value.normals);
        let indices = Indices::U32(value.indices);

        Self::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(indices)
    }
}
