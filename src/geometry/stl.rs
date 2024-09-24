use bevy::prelude::*;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;


#[derive(Default)]
pub struct StlLoader;

impl AssetLoader for StlLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = std::io::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await?;
        let mut bytes = std::io::Cursor::new(bytes);
        let stl_io::IndexedMesh { mut vertices, mut faces } = stl_io::read_stl(&mut bytes)?;

        let vertices: Vec<[f32; 3]> = vertices.drain(..)
            .map(|vertex| std::array::from_fn::<f32, 3, _>(|i| vertex[i]))
            .collect();
        let (indices, normals) = {
            let mut indices: Vec<u32> = Vec::with_capacity(3 * faces.len());
            let mut normals: Vec<[f32; 3]> = vec![[0.0_f32; 3]; vertices.len()];
            for face in faces.drain(..) {
                let normal: Vec3 = std::array::from_fn::<f32, 3, _>(|i| face.normal[i]).into();
                let normal = if normal.length() > 0.0 {
                    normal
                } else {
                    let v0: Vec3 = vertices[face.vertices[0]].into();
                    let v1: Vec3 = vertices[face.vertices[1]].into();
                    let v2: Vec3 = vertices[face.vertices[2]].into();
                    let u = v1 - v0;
                    let v = v2 - v0;
                    u.cross(v).normalize()
                };
                let normal: [f32; 3] = normal.into();

                face.vertices.iter()
                    .for_each(|index| {
                        indices.push(*index as u32);
                        normals[*index] = normal; // XXX Average the normal for shared vertices?
                    });
            }
            (indices, normals)
        };
        let vertices = VertexAttributeValues::Float32x3(vertices);
        let normals = VertexAttributeValues::Float32x3(normals);
        let indices = Indices::U32(indices);

        let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(indices);

        Ok(mesh)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["stl"];
        EXTENSIONS
    }
}
