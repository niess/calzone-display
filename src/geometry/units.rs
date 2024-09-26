pub trait Meters {
    fn meters(&self) -> f32;
}

impl Meters for f32 {
    fn meters(&self) -> f32 {
        *self * 1e-2
    }
}

impl Meters for f64 {
    fn meters(&self) -> f32 {
        (*self * 1e-2) as f32
    }
}
