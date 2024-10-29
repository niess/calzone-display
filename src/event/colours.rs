use bevy::color::LinearRgba;
use bevy::color::palettes::css;
use std::collections::HashMap;
use std::sync::LazyLock;


pub static COLOURS: LazyLock<HashMap<i32, LinearRgba>> = LazyLock::new(|| HashMap::from([
    ( 11,  LinearRgba::from(css::DARK_BLUE)),
    (-11,  LinearRgba::from(css::CORNFLOWER_BLUE)),
    ( 13,  LinearRgba::from(css::DARK_GREEN)),
    (-13,  LinearRgba::from(css::FOREST_GREEN)),
    ( 22,  LinearRgba::from(css::GOLD)),
]));
