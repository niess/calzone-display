use bevy::prelude::*;
use bevy::log::{Level, LogPlugin};
use bevy::render::{
    RenderPlugin, render_resource::WgpuLimits, settings::{RenderCreation, WgpuSettings}
};
use bevy::window::{ExitCondition::DontExit, PrimaryWindow};
use bevy::winit::{EventLoopProxyWrapper, WakeUp, WinitPlugin};
use bevy_rapier3d::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use super::display::DisplayPlugin;
use super::drone::DronePlugin;
use super::event::EventPlugin;
use super::geometry::GeometryPlugin;
use super::lighting::LightingPlugin;
use super::ui::UiPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub(crate) enum AppState {
    Display,
    #[default]
    Iddle,
}

#[derive(Component)]
pub(crate) struct Removable;

static EXIT: AtomicBool = AtomicBool::new(false);

pub fn set_exit() {
    EXIT.store(true, Ordering::Relaxed);
}

pub fn run() -> u8 {
    let winit = if cfg!(target_os = "macos") {
        WinitPlugin::<WakeUp>::default()
    } else {
        let mut winit = WinitPlugin::<WakeUp>::default();
        winit.run_on_any_thread = true;
        winit
    };

    let window = WindowPlugin {
        primary_window: None,
        exit_condition: DontExit,
        close_when_requested: true,
    };

    let log = if cfg!(debug_assertions) {
        LogPlugin {
            filter: "wgpu=error".to_string(),
            ..default()
        }
    } else {
        LogPlugin {
            level: Level::ERROR,
            filter: "".to_string(),
            ..default()
        }
    };

    let render = {
        let mut limits = WgpuLimits::default();
        fn maxify<T: Copy + PartialOrd>(x: &mut T, m: T) {
            if *x < m { *x = m }
        }
        maxify(&mut limits.max_sampled_textures_per_shader_stage, 16384);
        maxify(&mut limits.max_storage_buffers_per_shader_stage, 16);
        maxify(&mut limits.max_push_constant_size, 128);
        maxify(&mut limits.max_buffer_size, 4 * 1024_u64.pow(3));
        RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                constrained_limits: Some(limits),
                ..default()
            }),
            ..default()
        }
    };

    let mut app = App::new();
    let rc = app
        .add_plugins((
            DefaultPlugins.build()
                .set(log)
                .set(render)
                .set(window)
                .set(winit),
            RapierPhysicsPlugin::<NoUserData>::default(),
            DisplayPlugin,
            DronePlugin,
            EventPlugin,
            GeometryPlugin,
            LightingPlugin,
            UiPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(OnExit(AppState::Display), clear_all)
        .add_systems(Update, (
            iddle_system.run_if(in_state(AppState::Iddle)),
            display_system.run_if(in_state(AppState::Display)),
        ))
        .run();

    match rc {
        AppExit::Success => 0,
        AppExit::Error(rc) => rc.get(),
    }
}

fn iddle_system(
    mut commands: Commands,
    window: Query<&Window>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    event_loop_proxy: Res<EventLoopProxyWrapper<WakeUp>>,
    time: Res<Time>,
) {
    if GeometryPlugin::is_data() {
        if window.is_empty() {
            commands.spawn((
                Window {
                    title: "Calzone Display".to_owned(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .observe(on_window_closed);
            let _ = event_loop_proxy.send_event(WakeUp); // To trigger a winit redraw.
        }
        next_state.set(AppState::Display);
    } else if time.elapsed() > std::time::Duration::from_millis(100) {
        // Exiting too soon after startup might crash the gfx drivers (linux/nvidia).
        if EXIT.load(Ordering::Relaxed) {
            exit.write(AppExit::Success);
        }
    }
}

fn clear_all(
    entities: Query<Entity, With<Removable>>,
    mut commands: Commands,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

fn display_system(mut next_state: ResMut<NextState<AppState>>) {
    if GeometryPlugin::is_some() {
        next_state.set(AppState::Iddle); // Despawn the current display.
    }
}

fn on_window_closed(
    _trigger: Trigger<OnRemove, PrimaryWindow>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    next_state.set(AppState::Iddle); // Despawn the current display.
}
