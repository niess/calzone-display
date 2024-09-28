use bevy::prelude::*;
use bevy::color::palettes::css::*;
use super::data::VolumeInfo;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use super::units::Meters;


type Materials = HashMap<String, Handle<StandardMaterial>>;

static MATERIALS: LazyLock<Mutex<Materials>> = LazyLock::new(|| Mutex::new(Materials::new()));

#[derive(Bundle)]
pub struct VolumeBundle (PbrBundle);

impl VolumeBundle {
    pub fn new(
        volume: VolumeInfo,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let mesh = meshes.add(Mesh::from(volume.solid));
        let material = MATERIALS.lock().unwrap()
            .entry(volume.material)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    cull_mode: None,
                    ..default()
                })
            }).clone();
        let transform = Transform::from_xyz(
            volume.transform.translation[0].meters(),
            volume.transform.translation[1].meters(),
            volume.transform.translation[2].meters(),
        );
        // XXX Apply rotation as well.
        let pbr = PbrBundle { mesh, material, transform, ..default() };
        Self (pbr)
    }
}
