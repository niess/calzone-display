use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::data::Volume;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use super::units::Meters;


type Materials = HashMap<String, Handle<StandardMaterial>>;

static MATERIALS: LazyLock<Mutex<Materials>> = LazyLock::new(|| Mutex::new(Materials::new()));

#[derive(Bundle)]
pub struct VolumeBundle (PbrBundle);

impl VolumeBundle {
    pub fn new(
        volume: Volume,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let mesh = meshes.add(Mesh::from(volume.shape));
        let material = MATERIALS.lock().unwrap()
            .entry(volume.material)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    ..default()
                })
            }).clone();
        let transform = match volume.position {
            Some(r) => Transform::from_xyz(
                r[0].meters(),
                r[1].meters(),
                r[2].meters(),
            ), // XXX Apply rotation as well.
            None => match volume.rotation {
                Some(rotation) => unimplemented!(),
                None => Transform::default(),
            },
        };
        let pbr = PbrBundle { mesh, material, transform, ..default() };
        Self (pbr)
    }
}
