use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::input::mouse::MouseWheel;
use bevy::window::{PrimaryWindow, WindowResized};
use crate::app::AppState;


pub fn build(app: &mut App) {
    app.add_systems(Update, (
        Scroll::on_wheel,
        Scroll::on_resize,
    ).run_if(in_state(AppState::Display)));
}

#[derive(Component)]
pub struct Scroll;

impl Scroll {
    const SENSITIVITY: f32 = 15.0;
    const OFFSET: f32 = 40.0;

    pub fn spawn<'a>(commands: &'a mut Commands, window: &Window) -> EntityCommands<'a> {
        let height = (window.height() - Self::OFFSET).max(0.0);
        commands.spawn((
            Self,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    max_height: Val::Px(height),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
        ))
    }

    pub fn on_resize(
        mut events: EventReader<WindowResized>,
        mut styles: Query<&mut Style, With<Scroll>>,
    ) {
        if styles.is_empty() {
            return;
        }
        for event in events.read() {
            let height = (event.height - Self::OFFSET).max(0.0);
            for mut style in styles.iter_mut() {
                style.max_height = Val::Px(height);
            }
        }
    }

    pub fn on_wheel(
        mut wheels: EventReader<MouseWheel>,
        scrolls: Query<(&Node, &GlobalTransform, &Children), With<Scroll>>,
        window: Query<&mut Window, With<PrimaryWindow>>,
        mut children: Query<(&Node, &mut Style), With<Parent>>,
    ) {
        if window.is_empty() {
            return
        }
        let Some(cursor) = window.single().cursor_position() else { return };
        let mut delta = 0.0;
        for wheel in wheels.read() {
            delta += wheel.y;
        }
        if delta == 0.0 {
            return
        }
        for (node, transform, childs) in scrolls.iter() {
            let rect = node.logical_rect(transform);
            if rect.contains(cursor) {
                let max_height = node.size().y;
                for child in childs.iter() {
                    if let Ok((node, mut style)) = children.get_mut(*child) {
                        let height = node.size().y;
                        let top = match style.top {
                            Val::Px(v) => v,
                            _ => 0.0,
                        };
                        if (height <= max_height) && (top >= 0.0) {
                            continue
                        }
                        let min_top = top.min(max_height - height);
                        let new_top = (top + Self::SENSITIVITY * delta)
                            .clamp(min_top, 0.0);
                        if new_top != top {
                            style.top = Val::Px(new_top);
                        }
                    }
                }
            }
        }
    }
}
