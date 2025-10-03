use bevy::prelude::*;
use bevy::math::Affine3A;
use std::sync::OnceLock;

pub mod app;
mod drone;
mod display;
pub mod event;
pub mod geometry;
mod lighting;
mod ui;


static VIEW_TO_WORLD: OnceLock<Affine3A> = OnceLock::new();
static WORLD_TO_VIEW: OnceLock<Affine3A> = OnceLock::new();

const ROTATION_ANGLE: f32 = -std::f32::consts::FRAC_PI_2;

fn view_transform() -> Transform {
    Transform::from_rotation(Quat::from_rotation_x(ROTATION_ANGLE))
}

fn view_to_world() -> &'static Affine3A {
    VIEW_TO_WORLD.get_or_init(|| {
        Affine3A::from_rotation_x(-ROTATION_ANGLE)
    })
}

fn world_to_view() -> &'static Affine3A {
    WORLD_TO_VIEW.get_or_init(|| {
        Affine3A::from_rotation_x(ROTATION_ANGLE)
    })
}
