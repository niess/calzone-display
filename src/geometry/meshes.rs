use bevy::prelude::*;
use super::data::{BoxShape, CylinderShape, Shape, SphereShape};
use super::units::Meters;


impl From<Shape> for Mesh {
    fn from(value: Shape) -> Self {
        match value {
            Shape::Box(shape) => shape.into(),
            Shape::Cylinder(shape) => shape.into(),
            Shape::Envelope(_) => unimplemented!(),
            Shape::Sphere(shape) => shape.into(),
            Shape::Tessellation(_) => unimplemented!(),
        }
    }
}

impl From<BoxShape> for Mesh {
    fn from(value: BoxShape) -> Self {
        let size: Vec3 = std::array::from_fn(|i| value.size[i].meters()).into();
        Cuboid::from_size(size).into()
    }
}

impl From<CylinderShape> for Mesh {
    fn from(value: CylinderShape) -> Self {
        if value.thickness <= 0.0 {
            if value.section == [ 0.0, 360.0 ] {
                Cylinder::new(
                    value.radius.meters(),
                    value.length.meters(),
                )
                    .mesh()
                    .resolution(256)
                    .build()
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }
}

impl From<SphereShape> for Mesh {
    fn from(value: SphereShape) -> Self {
        if value.thickness <= 0.0 {
            if value.azimuth_section == [ 0.0, 360.0 ] && value.zenith_section == [1.0, 180.0] {
                Sphere::new(value.radius.meters())
                    .mesh()
                    .ico(7)
                    .unwrap_or_else(|err| panic!("{}", err))
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }
}
