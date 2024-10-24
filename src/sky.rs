use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_atmosphere::prelude::*;
use crate::app::AppState;
use crate::lighting::Sun;


// XXX Correct error on close.
// XXX Keyboard toggle for skybox.

pub struct SkyPlugin;

#[derive(Bundle)]
pub struct SkyBundle (SkyCamera, AtmosphereCamera, Camera3dBundle, RenderLayers);

#[derive(Component)]
pub struct SkyCamera;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AtmospherePlugin)
            .insert_resource(AtmosphereModel::new(Nishita::default()))
            .add_systems(Update, update_sky.run_if(in_state(AppState::Display)));
    }
}

fn update_sky(
    mut model: ResMut<AtmosphereModel>,
    sun: Res<Sun>,
) {
    if !sun.is_changed() {
        return
    }

    let position = sun.compute_position();
    let theta = position.zenith_angle.to_radians() as f32;
    let phi = (90.0 - position.azimuth).to_radians() as f32;

    *model = AtmosphereModel::new(Nishita {
        ray_origin: Vec3::new(0.0, 0.0, 6372E+03),
        sun_position: Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        ),
        ..default()
    });
}

const SKY_LAYER: usize = 1;

impl SkyBundle {
    pub fn new(fov: f32) -> Self {
        Self (
            SkyCamera,
            AtmosphereCamera {
                render_layers: Some(RenderLayers::layer(SKY_LAYER))
            },
            Camera3dBundle {
                camera: Camera {
                    order: -1,
                    ..default()
                },
                projection: PerspectiveProjection {
                    fov,
                    ..default()
                }.into(),
                ..default()
            },
            RenderLayers::layer(SKY_LAYER),
        )
    }
}
