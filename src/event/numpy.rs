use super::data::{CTrack, CVertex};
// PyO3 interface.
use pyo3::prelude::*;
use pyo3::{ffi, pyobject_native_type_extract, pyobject_native_type_named, PyTypeInfo};
use pyo3::sync::GILOnceCell;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::types::PyCapsule;
// Standard library.
use std::ffi::{c_char, c_int, c_uchar, c_void};
use std::marker::PhantomData;
use std::ops::Deref;


// ===============================================================================================
//
// Numpy array interface.
//
// ===============================================================================================

struct ArrayInterface {
    // Keep the capsule alive.
    #[allow(dead_code)]
    capsule: PyObject,
    // Type objects.
    dtype_track: PyObject,
    dtype_vertex: PyObject,
    type_ndarray: PyObject,
    // Functions.
    equiv_types: *const PyArray_EquivTypes,
}

#[allow(non_camel_case_types)]
pub type npy_intp = ffi::Py_intptr_t;

#[allow(non_camel_case_types)]
type PyArray_EquivTypes = extern "C" fn(
    type1: *mut ffi::PyObject,
    type2: *mut ffi::PyObject,
) -> c_uchar;

unsafe impl Send for ArrayInterface {}
unsafe impl Sync for ArrayInterface {}

static ARRAY_INTERFACE: GILOnceCell<ArrayInterface> = GILOnceCell::new();

fn api(py: Python) -> &ArrayInterface {
    ARRAY_INTERFACE
        .get(py)
        .expect("Numpy Array API not initialised")
}

pub fn initialise(py: Python) -> PyResult<()> {
    if let Some(_) = ARRAY_INTERFACE.get(py) {
        return Err(PyValueError::new_err("Numpy Array API already initialised"))
    }

    // Import interfaces.
    let numpy = PyModule::import_bound(py, "numpy")?;
    let capsule = PyModule::import_bound(py, "numpy.core.multiarray")?
        .getattr("_ARRAY_API")?;

    // Cache used dtypes, generated from numpy Python interface.
    let dtype = numpy.getattr("dtype")?;

    let dtype_track: PyObject = {
        let arg: [PyObject; 5] = [
            ("event", "u8").into_py(py),
            ("tid", "i4").into_py(py),
            ("parent", "i4").into_py(py),
            ("pid", "i4").into_py(py),
            ("creator", "S16").into_py(py),
        ];
        dtype
            .call1((arg, true))?
            .into_py(py)
    };

    let dtype_vertex: PyObject = {
        let arg: [PyObject; 7] = [
            ("event", "u8").into_py(py),
            ("tid", "i4").into_py(py),
            ("energy", "f8").into_py(py),
            ("position", "f8", 3).into_py(py),
            ("direction", "f8", 3).into_py(py),
            ("volume", "S16").into_py(py),
            ("process", "S16").into_py(py),
        ];
        dtype
            .call1((arg, true))?
            .into_py(py)
    };

    // Parse C interface.
    // See e.g. numpy/_core/code_generators/numpy_api.py for API mapping.
    let ptr = capsule
        .downcast::<PyCapsule>()?
        .pointer() as *const *const c_void;

    let object = |offset: isize| -> PyObject {
        unsafe {
            Py::<PyAny>::from_borrowed_ptr(py, *ptr.offset(offset) as *mut ffi::PyObject)
                .into_py(py)
        }
    };

    let function = |offset: isize| unsafe {
        ptr.offset(offset)
    };

    let api = ArrayInterface {
        capsule: capsule.into(),
        // Type objects.
        dtype_track,
        dtype_vertex,
        type_ndarray: object(2),
        // Functions.
        equiv_types:         function(182) as *const PyArray_EquivTypes,
    };

    // Initialise static data and return.
    match ARRAY_INTERFACE.set(py, api) {
        Err(_) => unreachable!(),
        Ok(_) => (),
    }
    Ok(())
}


// ===============================================================================================
//
// Generic (untyped) array.
//
// ===============================================================================================

#[repr(transparent)]
pub struct PyUntypedArray(PyAny);

#[repr(C)]
pub struct PyArrayObject {
    pub object: ffi::PyObject,
    pub data: *mut c_char,
    pub nd: c_int,
    pub dimensions: *mut npy_intp,
    pub strides: *mut npy_intp,
    pub base: *mut ffi::PyObject,
    pub descr: *mut ffi::PyObject,
    pub flags: c_int,
}

// Public interface.
impl PyUntypedArray {
    #[inline]
    pub fn dtype(&self) -> PyObject {
        unsafe { Py::<PyAny>::from_borrowed_ptr(self.py(), self.as_ptr()) }
    }

    #[inline]
    pub fn ndim(&self) -> usize {
        let obj: &PyArrayObject = self.as_ref();
        obj.nd as usize
    }

    #[inline]
    pub fn shape(&self) -> Vec<usize> {
        self.shape_slice()
            .iter()
            .map(|v| *v as usize)
            .collect()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.shape_slice()
            .iter()
            .product::<npy_intp>() as usize
    }
}

// Private interface.
impl PyUntypedArray {
    pub fn data(&self, index: usize) -> PyResult<*mut c_char> {
        let size = self.size();
        if index >= size {
            Err(PyIndexError::new_err(format!(
                "ndarray index out of range (expected an index in [0, {}), found {})",
                size,
                index
            )))
        } else {
            let offset = self.offset_of(index);
            let obj: &PyArrayObject = self.as_ref();
            let data = unsafe { obj.data.offset(offset as isize) };
            Ok(data)
        }
    }

    fn offset_of(&self, index: usize) -> isize {
        let shape = self.shape_slice();
        let strides = self.strides_slice();
        let n = shape.len();
        if n == 0 {
            0
        } else {
            let mut remainder = index;
            let mut offset = 0_isize;
            for i in (0..n).rev() {
                let m = shape[i] as usize;
                let j = remainder % m;
                remainder = (remainder - j) / m;
                offset += (j as isize) * strides[i];
            }
            offset
        }
    }

    #[inline]
    fn shape_slice(&self) -> &[npy_intp] {
        let obj: &PyArrayObject = self.as_ref();
        unsafe { std::slice::from_raw_parts(obj.dimensions, obj.nd as usize) }
    }

    #[inline]
    fn strides_slice(&self) -> &[npy_intp] {
        let obj: &PyArrayObject = self.as_ref();
        unsafe { std::slice::from_raw_parts(obj.strides, obj.nd as usize) }
    }
}

// Trait implementations.
impl AsRef<PyArrayObject> for PyUntypedArray {
    #[inline]
    fn as_ref(&self) -> &PyArrayObject {
        let ptr: *mut PyArrayObject = self.as_ptr().cast();
        unsafe { &*ptr }
    }
}

unsafe impl PyTypeInfo for PyUntypedArray {
    const NAME: &'static str = "PyUntypedArray";
    const MODULE: Option<&'static str> = Some("numpy");

    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject {
        api(py)
            .type_ndarray
            .as_ptr() as *mut ffi::PyTypeObject
    }
}

pyobject_native_type_named!(PyUntypedArray);

pyobject_native_type_extract!(PyUntypedArray);


// ===============================================================================================
//
// Typed array.
//
// ===============================================================================================

#[repr(transparent)]
pub struct PyArray<T>(PyUntypedArray, PhantomData<T>);

// Public interface.
impl<T> PyArray<T>
where
    T: Copy + Dtype,
{
    pub fn as_any(&self) -> &PyAny {
        &self.0
    }

    pub fn get(&self, index: usize) -> PyResult<T> {
        let data = self.data(index)?;
        let value = unsafe { *(data as *const T) };
        Ok(value)
    }

}

// Traits implementations.
impl<T> AsRef<PyArrayObject> for PyArray<T> {
    #[inline]
    fn as_ref(&self) -> &PyArrayObject {
        self.0.as_ref()
    }
}

impl<T> Deref for PyArray<T> {
    type Target = PyUntypedArray;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a, T> From<&'a PyArray<T>> for &'a PyUntypedArray {
    #[inline]
    fn from(ob: &'a PyArray<T>) -> &'a PyUntypedArray {
        unsafe { &*(ob as *const PyArray<T> as *const PyUntypedArray) }
    }
}

impl<'a, T> TryFrom<&'a PyUntypedArray> for &'a PyArray<T>
where
    T: Dtype,
{
    type Error = PyErr;

    #[inline]
    fn try_from(ob: &'a PyUntypedArray) -> Result<&'a PyArray<T>, Self::Error> {
        let dtype = T::dtype(ob.py())?;
        let array: &PyArrayObject = ob.as_ref();
        let mut same = array.descr as * const ffi::PyObject == dtype.as_ptr();
        if !same {
            let api = api(ob.py());
            let equiv_types = unsafe { *api.equiv_types };
            same = equiv_types(array.descr as * mut ffi::PyObject, dtype.as_ptr()) != 0;
        }
        if same {
            Ok(unsafe { &*(ob as *const PyUntypedArray as *const PyArray<T>) })
        } else {
            let expected: Bound<PyAny> = dtype.extract(ob.py()).unwrap();
            Err(PyTypeError::new_err(format!(
                "bad dtype (expected '{}', found '{}')",
                expected,
                unsafe { &*(array.descr as *mut PyAny) },
            )))
        }
    }
}

impl<'py, T> FromPyObject<'py> for &'py PyArray<T>
where
    T: Dtype,
{
    fn extract(obj: &'py PyAny) -> PyResult<Self> {
        let untyped: &PyUntypedArray = FromPyObject::extract(obj)?;
        let typed: &PyArray<T> = std::convert::TryFrom::try_from(untyped)?;
        Ok(typed)
    }
}

unsafe impl<T> PyNativeType for PyArray<T> {
    type AsRefSource = Self;
}


// ===============================================================================================
//
// D-types.
//
// ===============================================================================================

pub trait Dtype {
    fn dtype(py: Python) -> PyResult<PyObject>;
}

impl Dtype for CTrack {
    #[inline]
    fn dtype(py: Python) -> PyResult<PyObject> {
        Ok(api(py).dtype_track.clone_ref(py))
    }
}

impl Dtype for CVertex {
    #[inline]
    fn dtype(py: Python) -> PyResult<PyObject> {
        Ok(api(py).dtype_vertex.clone_ref(py))
    }
}

//================================================================================================
// Control flags for Numpy arrays.
//================================================================================================

pub enum PyArrayFlags {
    ReadOnly,
    ReadWrite,
}

impl PyArrayFlags {
    pub const C_CONTIGUOUS: c_int = 0x0001;
    pub const WRITEABLE:    c_int = 0x0400;
}

impl From<PyArrayFlags> for c_int {
    fn from(value: PyArrayFlags) -> Self {
        match value {
            PyArrayFlags::ReadOnly =>  PyArrayFlags::C_CONTIGUOUS,
            PyArrayFlags::ReadWrite => PyArrayFlags::C_CONTIGUOUS | PyArrayFlags::WRITEABLE,
        }
    }
}
