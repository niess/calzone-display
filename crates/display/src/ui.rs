use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy_simple_text_input::{TextInput, TextInputCursorPos, TextInputInactive,
    TextInputPlugin, TextInputSettings, TextInputSubmitEvent, TextInputSystem, TextInputTextColor,
    TextInputTextFont, TextInputValue};
use bevy::window::PrimaryWindow;
use crate::app::{AppState, Removable};
use crate::geometry::GeometrySet;

mod event;
mod geometry;
mod location;
mod meters;
mod nord;
mod scroll;
mod stats;

pub use event::UiEvent;
pub use location::LocationState;
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
        stats::build(app);
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
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                top, left, bottom, right,
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
    const FONT_SIZE: f32 = 14.0;

    fn new<'a>(
        title: &str,
        location: WindowLocation,
        commands: &'a mut Commands
    ) -> EntityCommands<'a> {
        let title = commands.spawn((
            Text(title.to_owned()),
            TextFont {
                font_size: Self::FONT_SIZE,
                ..default()
            },
            TextColor(NORD[6].into()),
        )).id();

        let mut capsule = commands.spawn((
            UiWindow,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(3.0), Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(NORD[2].into()),
        ));
        capsule.add_child(title);
        let capsule = capsule.id();

        let [top, left, bottom, right] = location.offsets();
        let position_type = match location {
            WindowLocation::Relative => PositionType::Relative,
            _ => PositionType::Absolute,
        };

        let mut window = commands.spawn((
            Node {
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
            BackgroundColor(NORD[1].into()),
            BorderColor(NORD[2].into()),
            BorderRadius::all(Val::Px(4.0)),
        ));
        window
            .insert(Removable)
            .add_child(capsule);
        window
    }
}

struct UiText;

impl UiText {
    pub const FONT_HEIGHT: f32 = 12.0;
    pub const FONT_ASPECT_RATIO: f32 = 0.5;

    const NORMAL: Srgba = NORD[4];
    const HOVERED: Srgba = NORD[7];
    const PRESSED: Srgba = NORD[1];

    #[inline]
    fn font_width() -> f32 {
        Self::FONT_HEIGHT * Self::FONT_ASPECT_RATIO
    }

    fn new_bundle(message: &str) -> impl Bundle {
        (
            Text(message.to_owned()),
            TextFont {
                font_size: Self::FONT_HEIGHT,
                ..default()
            },
            TextColor(Self::NORMAL.into()),
            Node {
                margin: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            },
        )
    }

    fn new_input(message: &str, width: f32) -> impl Bundle {
        (
            BorderColor(NORD[2].into()),
            Node {
                width: Val::Px(width),
                ..default()
            },
            TextInput,
            TextInputInactive(true),
            TextInputValue(message.to_owned()),
            TextInputSettings {
                retain_on_submit: true,
                ..default()
            },
            TextInputTextFont(TextFont {
                font_size: Self::FONT_HEIGHT,
                ..default()
            }),
            TextInputTextColor(TextColor(Self::NORMAL.into())),
        )
    }

    fn on_mouse_button(
        buttons: Res<ButtonInput<MouseButton>>,
        mut inputs: Query<(
            Entity, &ComputedNode, &GlobalTransform, &mut TextInputInactive, &TextInputValue,
            &mut TextInputCursorPos,
        )>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
        mut ev_input: EventWriter<TextInputSubmitEvent>,
    ) -> Result<()> {
        if window.is_empty() {
            return Ok(()); // The window might have been closed.
        }
        let window = window.single_mut()?;

        if buttons.just_pressed(MouseButton::Left) {
            if let Some(cursor) = window.cursor_position() {
                for (entity, node, transform, mut inactive, value, mut pos) in inputs.iter_mut() {
                    let rect = Rect::from_center_size(
                        transform.translation().xy(),
                        node.size,
                    );
                    if rect.contains(cursor) {
                        if inactive.0 {
                            inactive.0 = false;
                        }
                        pos.0 = ((cursor.x - rect.min.x) / Self::font_width() + 0.5) as usize;
                    } else if !inactive.0 {
                        let value = value.0.clone();
                        ev_input.write(TextInputSubmitEvent { entity, value });
                    }
                }
            }
        }

        Ok(())
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
            Button,
            Node {
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Self::new_bundle(message));
        })
        .id()
    }
}
