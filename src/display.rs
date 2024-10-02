use bevy::prelude::*;
use crate::geometry::Volume;
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

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DisplayMode>()
            .add_systems(Update, (on_keyboard, on_mode.after(on_keyboard)));
    }
}


fn on_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut alpha: ResMut<DisplayMode>,
) {
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        alpha.inc();
    }
    if keyboard_input.just_pressed(KeyCode::PageDown) {
        alpha.dec();
    }
}

fn on_mode(
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
