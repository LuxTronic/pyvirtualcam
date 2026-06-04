//! Rust core for pyvirtualcam.
//!
//! The crate owns backend implementations and shared format/conversion logic
//! while the Python package remains the compatibility API for existing users.
//! Linux `v4l2loopback` is the first Rust backend; other platforms continue to
//! use the existing native code until they can be ported and tested.

pub mod camera;
pub mod convert;
pub mod error;
pub mod formats;
pub mod fourcc;

#[cfg(target_os = "linux")]
pub mod linux;

pub use camera::{Camera, CameraBuilder};
pub use error::{Error, Result};
pub use formats::PixelFormat;
