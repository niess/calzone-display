use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy_simple_text_input::{TextInputBundle, TextInputCursorPos, TextInputInactive,
    TextInputPlugin, TextInputSettings, TextInputSubmitEvent, TextInputSystem, TextInputValue};
use bevy::window::PrimaryWindow;
use crate::app::{AppState, Removable};
use crate::geometry::GeometrySet;

mod event;
mod geometry;
mod location;
mod meters;
mod nord;
mod scroll;

pub use event::UiEvent;
pub use meters::Meters;
pub use nord::NORD;
pub use scroll::Scroll;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextInputPlugin)
            .init_state::<TextInputState>()
            .add_systems(OnEnter(AppState::Display), PrimaryMenu::spawn.after(GeometrySet))
            .add_systems(Update,
                (UiText::on_mouse_button, UiText::on_inactive_changed).chain()
                    .in_set(TextInputSet)
                    .run_if(in_state(AppState::Display))
                    .after(TextInputSystem)
            );
        event::build(app);
        geometry::build(app);
        location::build(app);
        scroll::build(app);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TextInputState {
    Active,
    #[default]
    Inactive,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextInputSet;

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
                align_self: AlignSelf::Start,
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
    pub const FONT_HEIGHT: f32 = 18.0;
    pub const FONT_ASPECT_RATIO: f32 = 0.5;

    const NORMAL: Srgba = NORD[4];
    const HOVERED: Srgba = NORD[7];
    const PRESSED: Srgba = NORD[1];

    #[inline]
    fn font_width() -> f32 {
        Self::FONT_HEIGHT * Self::FONT_ASPECT_RATIO
    }

    fn new_bundle(message: &str) -> TextBundle {
        TextBundle::from_section(
            message,
            TextStyle {
                font_size: Self::FONT_HEIGHT,
                color: Self::NORMAL.into(),
                ..default()
            }
        )
        .with_style(Style {
            margin: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        })
    }

    fn new_input(message: &str, width: f32) -> (NodeBundle, TextInputBundle) {
        (
            NodeBundle {
                style: Style {
                    width: Val::Px(width),
                    ..default()
                },
                border_color: NORD[2].into(),
                ..default()
            },
            TextInputBundle::default()
                .with_inactive(true)
                .with_value(message)
                .with_settings(TextInputSettings {
                    retain_on_submit: true,
                    ..default()
                })
                .with_text_style(TextStyle {
                    font_size: Self::FONT_HEIGHT,
                    color: Self::NORMAL.into(),
                    ..default()
                }),
        )
    }

    fn on_mouse_button(
        buttons: Res<ButtonInput<MouseButton>>,
        mut inputs: Query<(
            Entity, &Node, &GlobalTransform, &mut TextInputInactive, &TextInputValue,
            &mut TextInputCursorPos,
        )>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
        mut ev_input: EventWriter<TextInputSubmitEvent>,
    ) {
        if window.is_empty() {
            return; // The window might have been closed.
        }
        let window = window.single_mut();

        if buttons.just_pressed(MouseButton::Left) {
            if let Some(cursor) = window.cursor_position() {
                for (entity, node, transform, mut inactive, value, mut pos) in inputs.iter_mut() {
                    let rect = node.logical_rect(transform);
                    if rect.contains(cursor) {
                        if inactive.0 {
                            inactive.0 = false;
                        }
                        pos.0 = ((cursor.x - rect.min.x) / Self::font_width() + 0.5) as usize;
                    } else if !inactive.0 {
                        let value = value.0.clone();
                        ev_input.send(TextInputSubmitEvent { entity, value });
                    }
                }
            }
        }
    }

    fn on_inactive_changed(
        inactives: Query<&TextInputInactive, Changed<TextInputInactive>>,
        mut next_state: ResMut<NextState<TextInputState>>,
    ) {
        for inactive in inactives.iter() {
            if inactive.0 {
                next_state.set(TextInputState::Inactive);
            } else {
                next_state.set(TextInputState::Active);
            }
        }
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
