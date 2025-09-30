use bevy::prelude::*;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::render::view::RenderLayers;
use bevy_atmosphere::prelude::*;
use bevy_atmosphere::plugin::AtmosphereSkyBox;
use bevy_atmosphere::skybox;
use crate::app::AppState;
use crate::drone::Drone;
use crate::lighting::Sun;
use crate::ui::{TextInputSet, TextInputState};


pub struct SkyPlugin;

#[derive(Bundle)]
pub struct SkyBundle (SkyCamera, AtmosphereCamera, Camera3d, Camera, Projection, RenderLayers);

#[derive(Component)]
pub struct SkyCamera;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AtmospherePlugin)
            .insert_resource(AtmosphereModel::new(Nishita::default()))
            .add_systems(OnEnter(AppState::Display), add_skybox.after(Drone::spawn))
            .add_systems(Update, (
                on_keyboard
                    .after(TextInputSet)
                    .run_if(in_state(TextInputState::Inactive)),
                update_sky,
            ).run_if(in_state(AppState::Display)));
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

fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut visibility: Query<&mut Visibility, With<SkyCamera>>,
) -> Result<()> {
    let mut visibility = visibility.single_mut()?;

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            _ => unreachable!(),
        }
    }
    Ok(())
}

const SKY_LAYER: usize = 1;
const SKY_FAR: f32 = 1000.0;

fn add_skybox<'a>(
    mut commands: Commands,
    camera: Query<Entity, With<SkyCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<skybox::AtmosphereSkyBoxMaterial>,
) -> Result<()> {
    commands
        .entity(camera.single()?)
        .insert(Visibility::Visible)
        .with_children(|parent| {
            parent
                .spawn((
                    Mesh3d(meshes.add(skybox::mesh(SKY_FAR))),
                    MeshMaterial3d(material.0.clone()),
                    AtmosphereSkyBox,
                    NotShadowCaster,
                    NotShadowReceiver,
                    RenderLayers::layer(SKY_LAYER),
                ));
        });
    Ok(())
}

impl SkyBundle {
    pub fn new(fov: f32) -> Self {
        Self (
            SkyCamera,
            AtmosphereCamera {
                render_layers: Some(RenderLayers::layer(SKY_LAYER))
            },
            Camera3d::default(),
            Camera {
                order: -1,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                    far: SKY_FAR,
                    fov,
                    ..default()
            }),
            RenderLayers::layer(SKY_LAYER),
        )
    }
}
