use bevy::prelude::*;
use bevy::pbr::wireframe::Wireframe;
use crate::app::AppState;
use crate::geometry::{Plain, Transparent, Volume};
use crate::lighting::{Shadows, Sun};
use crate::ui::{TextInputSet, TextInputState};


pub struct DisplayPlugin;

#[derive(Clone, Copy, Default, Resource)]
#[repr(i32)]
enum DisplayMode {
    Blend,
    #[default]
    Opaque,
    Premultiplied,
    Guard,
}

#[derive(Clone, Copy, Default, Resource)]
#[repr(i32)]
enum WireframeMode {
    #[default]
    Disabled,
    Partial,
    Enabled,
    Guard,
}

#[derive(Resource)]
struct BlendSettings {
    alpha: f32,
}

#[derive(Resource)]
struct PremultipliedSettings {
    alpha: f32,
}

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DisplayMode>()
            .init_resource::<WireframeMode>()
            .init_resource::<BlendSettings>()
            .init_resource::<PremultipliedSettings>()
            .add_systems(Update, (
                on_keyboard
                    .after(TextInputSet)
                    .run_if(in_state(TextInputState::Inactive)),
                (on_display_mode, on_wireframe_mode).after(on_keyboard),
            ).run_if(in_state(AppState::Display)));
    }
}


fn on_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut display_mode: ResMut<DisplayMode>,
    mut wireframe_mode: ResMut<WireframeMode>,
    mut blend_settings: ResMut<BlendSettings>,
    mut premultiplied_settings: ResMut<PremultipliedSettings>,
) {
    if keys.just_pressed(KeyCode::PageUp) {
        if keys.pressed(KeyCode::ShiftLeft) {
            wireframe_mode.inc();
        } else {
            display_mode.inc();
        }
    }
    if keys.just_pressed(KeyCode::PageDown) {
        if keys.pressed(KeyCode::ShiftLeft) {
            wireframe_mode.dec();
        } else {
            display_mode.dec();
        }
    }

    const DELTA: f32 = 0.005;
    let mut delta = 0.0_f32;
    if keys.pressed(KeyCode::ArrowUp) {
        delta += DELTA;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        delta -= DELTA;
    }

    if delta != 0.0 {
        match *display_mode {
            DisplayMode::Blend => {
                blend_settings.alpha = (blend_settings.alpha + delta)
                    .clamp(0.0, 1.0);
            },
            DisplayMode::Premultiplied => {
                premultiplied_settings.alpha = (premultiplied_settings.alpha + delta)
                    .clamp(0.0, 1.0);
            },
            _ => (),
        }
    }
}

fn on_display_mode(
    mode: Res<DisplayMode>,
    blend_settings: Res<BlendSettings>,
    premultiplied_settings: Res<PremultipliedSettings>,
    handles: Query<&Handle<StandardMaterial>, With<Volume>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    sun: Res<Sun>,
) {
    if !(
        mode.is_changed() ||
        blend_settings.is_changed() ||
        premultiplied_settings.is_changed()
    ) {
        return
    }

    match *mode {
        DisplayMode::Blend => {
            for handle in handles.iter() {
                let material = materials.get_mut(handle).unwrap();
                material.alpha_mode = AlphaMode::Blend;
                material.base_color.set_alpha(blend_settings.alpha);
            }
            Shadows::disable(&mut commands, &sun);
        },
        DisplayMode::Opaque => {
            for handle in handles.iter() {
                let material = materials.get_mut(handle).unwrap();
                material.alpha_mode = AlphaMode::Opaque;
                material.base_color.set_alpha(1.0);
            }
            Shadows::enable(&mut commands, &sun);
        },
        DisplayMode::Premultiplied => {
            for handle in handles.iter() {
                let material = materials.get_mut(handle).unwrap();
                material.alpha_mode = AlphaMode::Premultiplied;
                material.base_color.set_alpha(premultiplied_settings.alpha);
            }
            Shadows::disable(&mut commands, &sun);
        },
        _ => unreachable!(),
    }
}

fn on_wireframe_mode(
    mode: Res<WireframeMode>,
    standard_entities: Query<Entity, With<Plain>>,
    wired_entities: Query<Entity, With<Transparent>>,
    mut commands: Commands,
) {
    if !mode.is_changed() {
        return
    }

    match *mode {
        WireframeMode::Disabled => {
            for entity in standard_entities.iter().chain(wired_entities.iter()) {
                commands
                    .entity(entity)
                    .remove::<Wireframe>();
            }
        },
        WireframeMode::Partial => {
            for entity in standard_entities.iter() {
                commands
                    .entity(entity)
                    .remove::<Wireframe>();
            }
            for entity in wired_entities.iter() {
                commands
                    .entity(entity)
                    .insert(Wireframe);
            }
        },
        WireframeMode::Enabled => {
            for entity in standard_entities.iter().chain(wired_entities.iter()) {
                commands
                    .entity(entity)
                    .insert(Wireframe);
            }
        },
        _ => unreachable!(),
    }
}

impl DisplayMode {
    fn dec(&mut self) {
        *self = ((*self as i32) - 1)
            .rem_euclid(Self::Guard as i32)
            .into();
    }

    fn inc(&mut self) {
        *self = ((*self as i32) + 1)
            .rem_euclid(Self::Guard as i32)
            .into();
    }
}

impl From<i32> for DisplayMode {
    fn from(value: i32) -> Self {
        if value == (Self::Blend as i32) {
            Self::Blend
        } else if value == (Self::Opaque as i32) {
            Self::Opaque
        } else if value == (Self::Premultiplied as i32) {
            Self::Premultiplied
        } else {
            unreachable!()
        }
    }
}

impl WireframeMode {
    fn dec(&mut self) {
        *self = ((*self as i32) - 1)
            .rem_euclid(Self::Guard as i32)
            .into();
    }

    fn inc(&mut self) {
        *self = ((*self as i32) + 1)
            .rem_euclid(Self::Guard as i32)
            .into();
    }
}

impl From<i32> for WireframeMode {
    fn from(value: i32) -> Self {
        if value == (Self::Disabled as i32) {
            Self::Disabled
        } else if value == (Self::Partial as i32) {
            Self::Partial
        } else if value == (Self::Enabled as i32) {
            Self::Enabled
        } else {
            unreachable!()
        }
    }
}

impl Default for BlendSettings {
    fn default() -> Self {
        Self { alpha: 0.25 }
    }
}

impl Default for PremultipliedSettings {
    fn default() -> Self {
        Self { alpha: 0.25 }
    }
}
