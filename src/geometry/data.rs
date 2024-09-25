use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::PyBytes;
use rmp_serde::Deserializer;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct GeometryData {
    definition: GeometryDefinition,
    algorithm: Algorithm,
}

impl GeometryData {
    pub fn new(py: Python, path: &str) -> PyResult<Self> {
        let bytes = py.import_bound("calzone")
            .and_then(|x| x.getattr("GeometryBuilder"))
            .and_then(|x| x.call1((path,)))
            .and_then(|x| x.getattr("__getstate__"))
            .and_then(|x| x.call0())?;
        let bytes = bytes.downcast::<PyBytes>()?;

        let mut deserializer = Deserializer::new(bytes.as_bytes());
        Deserialize::deserialize(&mut deserializer)
            .map_err(|err| {
                let msg = format!("{}", err);
                PyTypeError::new_err(msg)
            })
    }
}


#[derive(Deserialize)]
enum Algorithm {
    Bvh,
    Geant4,
}

#[derive(Deserialize)]
struct GeometryDefinition {
    volume: Box<Volume>,
    materials: Option<MaterialsDefinition>,
}

#[derive(Deserialize)]
struct Volume {
    name: String,
    material: String,
    shape: Shape,
    position: Option<[f64; 3]>,
    rotation: Option<[[f64; 3]; 3]>,
    volumes: Vec<Volume>,
    overlaps: Vec<[String; 2]>,
    roles: Roles,
    subtract: Vec<String>,
    materials: Option<MaterialsDefinition>,
}

#[derive(Deserialize)]
enum Shape {
    Box(BoxShape),
    Cylinder(CylinderShape),
    Envelope(EnvelopeShape),
    Sphere(SphereShape),
    Tessellation(TessellatedShape),
}

#[derive(Deserialize)]
#[repr(C)]
struct BoxShape {
    size: [f64; 3],
}

#[derive(Deserialize)]
#[repr(C)]
struct CylinderShape {
    radius: f64,
    length: f64,
    thickness: f64,
    section: [f64; 2],
}

#[derive(Deserialize)]
#[repr(C)]
struct EnvelopeShape {
    shape: ShapeType,
    safety: f64,
}

#[derive(Deserialize)]
#[repr(i32)]
enum ShapeType {
    Box,
    Cylinder,
    Envelope,
    Sphere,
    Tessellation,
}

#[derive(Deserialize)]
#[repr(C)]
struct SphereShape {
    radius: f64,
    thickness: f64,
    azimuth_section: [f64; 2],
    zenith_section: [f64; 2],
}

#[derive(Deserialize)]
#[repr(C)]
struct TessellatedShape {
    facets: Vec<f32>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct Action (u32);

#[derive(Deserialize)]
#[repr(C)]
struct Roles {
    ingoing: Action,
    outgoing: Action,
    deposits: Action,
}

#[derive(Deserialize)]
struct MaterialsDefinition {
    elements: Vec<Element>,
    molecules: Vec<Molecule>,
    mixtures: Vec<Mixture>,
}

#[derive(Deserialize)]
#[repr(C)]
#[allow(non_snake_case)]
struct Element {
    name: String,
    symbol: String,
    Z: f64,
    A: f64,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct G4State (u32);

#[derive(Deserialize)]
#[repr(C)]
struct MaterialProperties {
    name: String,
    density: f64,
    state: G4State,
}

#[derive(Deserialize)]
#[repr(C)]
struct Mixture {
    properties: MaterialProperties,
    components: Vec<MixtureComponent>,
}

#[derive(Deserialize)]
#[repr(C)]
struct MixtureComponent {
    name: String,
    weight: f64,
}

#[derive(Deserialize)]
#[repr(C)]
struct Molecule {
    properties: MaterialProperties,
    components: Vec<MoleculeComponent>,
}

#[derive(Deserialize)]
#[repr(C)]
struct MoleculeComponent {
    name: String,
    weight: u32,
}
