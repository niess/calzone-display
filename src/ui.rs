use bevy::prelude::*;
use super::geometry::GeometrySet;

mod geometry;
mod nord;

pub use geometry::TargetEvent;
pub use nord::NORD;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui.after(GeometrySet));
        geometry::build(app);
    }
}

#[derive(Component)]
struct UiMenu;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        UiMenu,
        NodeBundle {
            style: Style {
                width: Val::Auto,
                height: Val::Auto,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            ..default()
        },
    ));
}

impl UiMenu {
    fn add_column<T>(
        component: T,
        commands: &mut Commands,
        ui: Query<Entity, With<Self>>,
    ) -> Entity
    where
        T: Component,
    {
        let column = commands.spawn((
            component,
            NodeBundle {
                style: Style {
                    width: Val::Auto,
                    height: Val::Auto,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: NORD[1].into(),
                border_color: NORD[2].into(),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
        )).id();
        commands
            .entity(ui.single())
            .add_child(column);
        column
    }
}

struct UiText;

impl UiText {
    const NORMAL: Srgba = NORD[4];
    const HOVERED: Srgba = NORD[7];
    const PRESSED: Srgba = NORD[1];

    fn new_bundle(message: &str) -> TextBundle {
        TextBundle::from_section(
            message,
            TextStyle {
                font_size: 18.0,
                color: Self::NORMAL.into(),
                ..default()
            }
        )
        .with_style(Style {
            margin: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        })
    }

    fn spawn_button<T>(
        component: T,
        message: &str,
        commands: &mut Commands,
    ) -> Entity
    where
        T: Component,
    {
        commands.spawn((
            component,
            ButtonBundle {
                style: Style {
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Self::new_bundle(message));
        })
        .id()
    }
}
