use bevy::prelude::*;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use serde::{Deserialize, Serialize};
use super::meshes::MeshData;


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
    interpolate_normal: bool,
}

impl Default for StlLoaderSettings {
    fn default() -> Self {
        Self {
            compute_normal: true,
            interpolate_normal: false,
        }
    }
}

impl StlLoaderSettings {
    fn build(&self, mesh: stl_io::IndexedMesh) -> Mesh {
        if self.interpolate_normal {
            self.build_interpolate(mesh)
        } else {
            self.build_face_normal(mesh)
        }
    }

    fn build_face_normal(&self, mesh: stl_io::IndexedMesh) -> Mesh {
        let stl_io::IndexedMesh { vertices: stl_vertices, mut faces } = mesh;
        let n = 3 * faces.len();
        let mut vertices = Vec::with_capacity(n); // Vertices are duplicated in order to properly
        let mut normals = Vec::with_capacity(n);  // apply faces normals.
        let mut indices = Vec::with_capacity(n);

        for (i, face) in faces.drain(..).enumerate() {
            let v: [[f32; 3]; 3] = std::array::from_fn(|j| {
                let v = stl_vertices[face.vertices[j]];
                std::array::from_fn(|k| v[k])
            });

            let normal = self.get_normal(&face, &v);

            for j in 0..3 {
                vertices.push(v[j]);
                indices.push((3 * i + j) as u32);
                normals.push(normal);
            }
        }

        MeshData { vertices, normals, indices }.into()
    }

    fn build_interpolate(&self, mesh: stl_io::IndexedMesh) -> Mesh {
        let stl_io::IndexedMesh { mut vertices, mut faces } = mesh;

        let vertices: Vec<[f32; 3]> = vertices.drain(..)
             .map(|vertex| std::array::from_fn::<f32, 3, _>(|i| vertex[i]))
             .collect();

        let (indices, mut normals) = {
            let mut indices: Vec<u32> = Vec::with_capacity(3 * faces.len());
            let mut normals = vec![Vec3::ZERO; vertices.len()];
            for face in faces.drain(..) {
                let v: [Vec3; 3] = std::array::from_fn(|i| vertices[face.vertices[i]].into());
                let normal = self.get_normal(&face, &v);
                let c = (v[0] + v[1] + v[2]) / 3.0;
                face.vertices.iter()
                    .enumerate()
                    .for_each(|(i, index)| {
                        indices.push(*index as u32);
                        let w = 1.0 / (v[i] - c).length();
                        normals[*index] += w * normal;
                    });
            }
            (indices, normals)
        };

        let normals = {
            let mut n: Vec<[f32; 3]> = Vec::with_capacity(normals.len());
            for normal in normals.drain(..) {
                let normal: [f32; 3] = normal.normalize().into();
                n.push(normal);
            }
            n
        };

        MeshData { vertices, normals, indices }.into()
    }

    #[inline]
    fn get_normal<T> (
        &self,
        face: &stl_io::IndexedTriangle,
        vertices: &[T; 3],
    ) -> T
    where
        T: Copy + From<[f32; 3]> + From<Vec3>,
        Vec3: From<T>,
    {
        let normal: T = std::array::from_fn::<f32, 3, _>(|i| face.normal[i]).into();
        if self.compute_normal {
            let normal: Vec3 = normal.into();
            let normal = if normal.length() > 0.0 {
                normal
            } else {
                MeshData::compute_normal(vertices)
            };
            normal.into()
        } else {
            normal
        }
    }
}
