use data::event::{CTrack, CVertex, Events};
use pyo3::prelude::*;
use super::numpy::{Dtype, PyArray};

struct Iter<'a, T>
where
    T: Dtype,
{
    array: &'a PyArray<T>,
    size: usize,
    index: usize,
}

impl<'a, T> Iter<'a, T>
where
    T: Dtype,
{
    fn new(array: &'a PyArray<T>) -> Self {
        let size = array.size();
        let index = 0;
        Self { array, size, index }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Clone + Copy + Dtype,
{
    type Item = PyResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let value = self.array.get(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

pub fn parse(data: &Bound<PyAny>) -> PyResult<()> {
    let tracks = data.getattr("tracks")?;
    let tracks: &PyArray<CTrack> = tracks.extract()?;
    let vertices = data.getattr("vertices")?;
    let vertices: &PyArray<CVertex> = vertices.extract()?;

    let tracks = Iter::new(tracks);
    let vertices = Iter::new(vertices);
    let events = Events::new(tracks, vertices)?;

    #[cfg(feature = "ipc")]
    crate::ipc::send_events(data.py(), events)?;

    #[cfg(feature = "thread")]
    display::event::set(events);

    Ok(())
}
