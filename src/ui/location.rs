use bevy::prelude::*;
use bevy_simple_text_input::{TextInputInactive, TextInputSubmitEvent, TextInputValue};
use crate::app::AppState;
use crate::lighting::Sun;
use super::{UiRoot, UiText, UiWindow};


pub fn build(app: &mut App) {
    app
        .add_systems(OnEnter(AppState::Display), setup_panel)
        .add_systems(Update,
            on_submit
                .run_if(in_state(AppState::Display))
                .after(UiText::on_mouse_button)
        );
}

#[derive(Component)]
enum Property {
    Day,
    Latitude,
    Month,
    Time,
    Year,
}

fn setup_panel(
    mut commands: Commands,
    sun: Res<Sun>,
) {
    const LABELS: [&'static str; 5] = [ "latitude", "time", "day", "month", "year" ];
    const UNITS: [&'static str; 5] = [ "deg", " h ", "", "", "" ];

    let labels = LABELS.map(|label| commands.spawn(UiText::new_bundle(label)).id());
    let units = UNITS.map(|label| commands.spawn(UiText::new_bundle(label)).id());

    fn format<T>(property: Property, value: T) -> (Property, String)
    where
        T: std::fmt::Display
    {
        let value = property.format(value);
        (property, value)
    }

    let values = [
        format(Property::Latitude, sun.latitude),
        format(Property::Time, sun.time),
        format(Property::Day, sun.day),
        format(Property::Month, sun.month),
        format(Property::Year, sun.year),
    ];
    let values = values.map(
        |(property, value)| commands.spawn((
            UiText::new_input(&value, (7.5 * UiText::font_width()).round()),
            property,
        )).id()
    );

    let columns = [labels, values, units];
    let columns = columns.map(|column| {
        let mut entity = commands.spawn(
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            }
        );
        entity.push_children(&column);
        entity.id()
    });

    let mut content = commands.spawn(
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },
    );
    content.push_children(&columns);
    let content = content.id();

    let mut panel = UiWindow::new("Location", super::WindowLocation::BottomRight, &mut commands);
    panel.add_child(content);
    panel.insert(UiRoot);
}

fn on_submit(
    mut events: EventReader<TextInputSubmitEvent>,
    mut inputs: Query<(&Property, &mut TextInputInactive, &mut TextInputValue)>,
    mut sun: ResMut<Sun>,
) {
    for event in events.read() {
        let Ok((property, mut inactive, mut input_value)) = inputs.get_mut(event.entity)
            else { continue };
        inactive.0 = true;
        property.update(&event.value, &mut sun, &mut input_value);
    }
}

impl Property {
    fn update(&self, new_value: &str, sun: &mut Sun, input_value: &mut TextInputValue) {
        match self {
            Self::Day => self.try_update_or_reset(
                new_value, &mut sun.day, Some(1), Some(31), input_value
            ),
            Self::Latitude => self.try_update_or_reset(
                new_value, &mut sun.latitude, Some(-90.0), Some(90.0), input_value
            ),
            Self::Month => self.try_update_or_reset(
                new_value, &mut sun.month, Some(1), Some(12), input_value
            ),
            Self::Time => self.try_update_or_reset(
                new_value, &mut sun.time, Some(0.0), Some(24.0), input_value
            ),
            Self::Year => self.try_update_or_reset(
                new_value, &mut sun.year, None, None, input_value
            ),
        }
    }

    fn format<T: std::fmt::Display>(&self, value: T) -> String {
        match self {
            Self::Day => format!("{:7}", value),
            Self::Latitude => format!("{:7.2}", value),
            Self::Month => format!("{:7}", value),
            Self::Time => format!("{:7}", value),
            Self::Year => format!("{:7}", value),
        }
    }

    fn try_update<T>(
        new_value: &str,
        current_value: &mut T,
        min: Option<T>,
        max: Option<T>,
    ) -> Option<T>
    where
        T: Copy + std::cmp::PartialOrd + std::str::FromStr + std::fmt::Display,
    {
        let v = T::from_str(new_value.trim()).ok()?;
        if let Some(min) = min {
            if v < min { return None }
        }
        if let Some(max) = max {
            if v > max { return None }
        }
        *current_value = v;
        return Some(v)
    }

    fn try_update_or_reset<T>(
        &self,
        new_value: &str,
        current_value: &mut T,
        min: Option<T>,
        max: Option<T>,
        input_value: &mut TextInputValue,
    )
    where
        T: Copy + std::cmp::PartialOrd + std::fmt::Display + std::str::FromStr,
    {
        let value = Self::try_update::<T>(new_value, current_value, min, max)
            .unwrap_or_else(|| *current_value);
        input_value.0 = self.format::<T>(value);
    }
}
