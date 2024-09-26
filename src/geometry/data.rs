use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::PyBytes;
use rmp_serde::Deserializer;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct GeometryData {
    pub definition: GeometryDefinition,
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
pub struct GeometryDefinition {
    pub volume: Volume,
    materials: Option<MaterialsDefinition>,
}

#[derive(Deserialize)]
pub struct Volume {
    name: String,
    pub material: String,
    pub shape: Shape,
    pub position: Option<[f64; 3]>,
    pub rotation: Option<[[f64; 3]; 3]>,
    pub volumes: Vec<Volume>,
    overlaps: Vec<[String; 2]>,
    roles: Roles,
    subtract: Vec<String>,
    materials: Option<MaterialsDefinition>,
}

#[derive(Deserialize)]
pub enum Shape {
    Box(BoxShape),
    Cylinder(CylinderShape),
    Envelope(EnvelopeShape),
    Sphere(SphereShape),
    Tessellation(TessellatedShape),
}

#[derive(Deserialize)]
pub struct BoxShape {
    pub size: [f64; 3],
}

#[derive(Deserialize)]
pub struct CylinderShape {
    pub radius: f64,
    pub length: f64,
    pub thickness: f64,
    pub section: [f64; 2],
}

#[derive(Deserialize)]
pub struct EnvelopeShape {
    pub shape: ShapeType,
    pub safety: f64,
}

#[derive(Deserialize)]
#[repr(i32)]
pub enum ShapeType {
    Box,
    Cylinder,
    Envelope,
    Sphere,
    Tessellation,
}

#[derive(Deserialize)]
pub struct SphereShape {
    pub radius: f64,
    pub thickness: f64,
    pub azimuth_section: [f64; 2],
    pub zenith_section: [f64; 2],
}

#[derive(Deserialize)]
pub struct TessellatedShape {
    pub facets: Vec<f32>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct Action (u32);

#[derive(Deserialize)]
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
struct Mixture {
    properties: MaterialProperties,
    components: Vec<MixtureComponent>,
}

#[derive(Deserialize)]
struct MixtureComponent {
    name: String,
    weight: f64,
}

#[derive(Deserialize)]
struct Molecule {
    properties: MaterialProperties,
    components: Vec<MoleculeComponent>,
}

#[derive(Deserialize)]
struct MoleculeComponent {
    name: String,
    weight: u32,
}
