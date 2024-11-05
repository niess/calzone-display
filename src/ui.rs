use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use crate::app::{AppState, Removable};
use crate::geometry::GeometrySet;

mod event;
mod geometry;
mod meters;
mod nord;

pub use event::UiEvent;
pub use meters::Meters;
pub use nord::NORD;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Display), PrimaryMenu::spawn.after(GeometrySet));
        event::build(app);
        geometry::build(app);
    }
}

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct PrimaryMenu;

impl PrimaryMenu {
    fn spawn(mut commands: Commands) {
        let [top, left, bottom, right] = WindowLocation::TopLeft.offsets();
        commands.spawn((
            PrimaryMenu,
            UiRoot,
            Removable,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    top, left, bottom, right,
                    ..default()
                },
                ..default()
            },
        ));
    }
}

#[derive(Component)]
struct UiWindow;

#[allow(dead_code)]
enum WindowLocation {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Relative,
    Cursor(Vec2),
}

impl WindowLocation {
    const OFFSET: Val = Val::Px(5.0);

    pub fn offsets(&self) -> [Val; 4] {
        match self {
            Self::TopLeft => [Self::OFFSET, Self::OFFSET, Val::Auto, Val::Auto],
            Self::TopRight => [Self::OFFSET, Val::Auto, Val::Auto, Self::OFFSET],
            Self::BottomLeft => [Val::Auto, Self::OFFSET, Self::OFFSET, Val::Auto],
            Self::BottomRight => [Val::Auto, Val::Auto, Self::OFFSET, Self::OFFSET],
            Self::Relative => [Val::Auto, Val::Auto, Val::Auto, Val::Auto],
            Self::Cursor(cursor) => [
                Val::Px(cursor.y + 12.0),
                Val::Px(cursor.x + 12.0),
                Val::Auto,
                Val::Auto,
            ],
        }
    }
}

impl UiWindow {
    fn new<'a>(
        title: &str,
        location: WindowLocation,
        commands: &'a mut Commands
    ) -> EntityCommands<'a> {
        let title = commands.spawn(
            TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 18.0,
                    color: NORD[6].into(),
                    ..default()
                }
            )
        ).id();

        let mut capsule = commands.spawn((
                UiWindow,
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_self: AlignSelf::Stretch,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(3.0), Val::Px(5.0)),
                        ..default()
                    },
                    background_color: NORD[2].into(),
                    ..default()
                },
        ));
        capsule.add_child(title);
        let capsule = capsule.id();

        let [top, left, bottom, right] = location.offsets();
        let position_type = match location {
            WindowLocation::Relative => PositionType::Relative,
            _ => PositionType::Absolute,
        };

        let mut window = commands.spawn(NodeBundle {
            style: Style {
                position_type,
                top,
                left,
                bottom,
                right,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: NORD[1].into(),
            border_color: NORD[2].into(),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        });
        window
            .insert(Removable)
            .add_child(capsule);
        window
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
