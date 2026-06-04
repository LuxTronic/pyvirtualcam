use numpy::PyReadonlyArray1;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};

use pyvirtualcam_core::linux::V4l2LoopbackCamera;
use pyvirtualcam_core::Error;

#[pyclass]
struct Camera {
    camera: V4l2LoopbackCamera,
}

#[pymethods]
impl Camera {
    #[new]
    #[pyo3(signature = (*, width, height, fps, fourcc, device=None))]
    fn new(
        width: u32,
        height: u32,
        fps: f64,
        fourcc: u32,
        device: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        if !fps.is_finite() || fps <= 0.0 {
            return Err(PyValueError::new_err("fps must be a positive finite number"));
        }
        // fps is handled by the Python Camera wrapper for pacing; the Linux
        // v4l2loopback backend does not configure device frame rate (same as
        // the legacy C++ backend).
        let _ = fps;
        let devices = parse_devices(device)?;
        let camera = V4l2LoopbackCamera::new(width, height, fourcc, devices).map_err(to_py_err)?;
        Ok(Self { camera })
    }

    fn close(&mut self) {
        self.camera.stop();
    }

    fn send(&mut self, frame: PyReadonlyArray1<'_, u8>) -> PyResult<()> {
        let frame = frame.as_slice()?;
        self.camera.send(frame).map_err(to_py_err)
    }

    fn device(&self) -> String {
        self.camera.device()
    }

    fn native_fourcc(&self) -> u32 {
        self.camera.native_fourcc()
    }
}

#[pymodule]
fn _native_linux_v4l2loopback(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Camera>()?;
    Ok(())
}

fn parse_devices(device: Option<&Bound<'_, PyAny>>) -> PyResult<Option<Vec<String>>> {
    let Some(device) = device else {
        return Ok(None);
    };
    if device.is_none() {
        return Ok(None);
    }

    if device.downcast::<PyString>().is_ok() {
        return Ok(Some(vec![to_string_like(device)?]));
    }

    if let Ok(sequence) = device.downcast::<PySequence>() {
        let len = sequence.len()?;
        let mut devices = Vec::with_capacity(len);
        for index in 0..len {
            let item = sequence.get_item(index)?;
            devices.push(to_string_like(&item)?);
        }
        return Ok(Some(devices));
    }

    Ok(Some(vec![to_string_like(device)?]))
}

fn to_string_like(value: &Bound<'_, PyAny>) -> PyResult<String> {
    Ok(value.str()?.to_str()?.to_owned())
}

fn to_py_err(error: Error) -> PyErr {
    match error {
        Error::InvalidArgument(message) => PyValueError::new_err(message),
        Error::Runtime(message) => PyRuntimeError::new_err(message),
        Error::Io { context, source } => PyRuntimeError::new_err(format!("{context}: {source}")),
    }
}
