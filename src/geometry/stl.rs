use bevy::prelude::*;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use serde::{Deserialize, Serialize};


#[derive(Default)]
pub struct StlLoader;

impl AssetLoader for StlLoader {
    type Asset = Mesh;
    type Settings = StlLoaderSettings;
    type Error = std::io::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await?;
        let mut bytes = std::io::Cursor::new(bytes);
        let mesh = stl_io::read_stl(&mut bytes)?;
        let mesh = settings.build(mesh);
        Ok(mesh)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["stl"];
        EXTENSIONS
    }
}

#[derive(Deserialize, Serialize)]
pub struct StlLoaderSettings {
    compute_normal: bool,
}

impl Default for StlLoaderSettings {
    fn default() -> Self {
        Self {
            compute_normal: true,
        }
    }
}

impl StlLoaderSettings {
    fn build(&self, mesh: stl_io::IndexedMesh) -> Mesh {
        let stl_io::IndexedMesh { vertices: stl_vertices, mut faces } = mesh;
        let n = 3 * faces.len();
        let mut vertices = Vec::with_capacity(n); // Vertices are duplicated in order to properly
        let mut normals = Vec::with_capacity(n);  // apply faces normals.
        let mut indices = Vec::with_capacity(n);

        for (i, face) in faces.drain(..).enumerate() {
            let v: [[f32; 3]; 3] = {
                let mut indices = face.vertices.iter();
                std::array::from_fn(|_| {
                    let v = stl_vertices[*indices.next().unwrap()];
                    std::array::from_fn(|i| v[i])
                })
            };

            let normal = self.get_normal(&face, &v[0], &v[1], &v[2]);

            for j in 0..3 {
                vertices.push(v[j]);
                indices.push((3 * i + j) as u32);
                normals.push(normal);
            }
        }

        let vertices = VertexAttributeValues::Float32x3(vertices);
        let normals = VertexAttributeValues::Float32x3(normals);
        let indices = Indices::U32(indices);

        Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(indices)
    }

    #[inline]
    fn get_normal(
        &self,
        face: &stl_io::IndexedTriangle,
        v0: &[f32; 3],
        v1: &[f32; 3],
        v2: &[f32; 3],
    ) -> [f32; 3] {
        let normal = std::array::from_fn::<f32, 3, _>(|i| face.normal[i]);
        if self.compute_normal {
            let normal: Vec3 = normal.into();
            let normal = if normal.length() > 0.0 {
                normal
            } else {
                let v0 = Vec3::from(*v0);
                let v1 = Vec3::from(*v1);
                let v2 = Vec3::from(*v2);
                (v1 - v0).cross(v2 - v0).normalize()
            };
            normal.into()
        } else {
            normal
        }
    }
}
