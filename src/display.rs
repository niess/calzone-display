use bevy::prelude::*;
use bevy::pbr::wireframe::Wireframe;
use crate::geometry::{Plain, Transparent, Volume};
use crate::lighting::{Shadows, Sun};


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

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DisplayMode>()
            .init_resource::<WireframeMode>()
            .add_systems(Update, (
                on_keyboard,
                (on_display_mode, on_wireframe_mode).after(on_keyboard),
            ));
    }
}


fn on_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut display_mode: ResMut<DisplayMode>,
    mut wireframe_mode: ResMut<WireframeMode>,
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
}

fn on_display_mode(
    mode: Res<DisplayMode>,
    handles: Query<&Handle<StandardMaterial>, With<Volume>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    sun: Res<Sun>,
) {
    if !mode.is_changed() {
        return
    }

    match *mode {
        DisplayMode::Blend => {
            for handle in handles.iter() {
                let material = materials.get_mut(handle).unwrap();
                material.alpha_mode = AlphaMode::Blend;
                material.base_color.set_alpha(0.33);
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
                material.base_color.set_alpha(0.0);
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
