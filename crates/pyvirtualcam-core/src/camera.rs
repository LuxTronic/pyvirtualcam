use crate::{PixelFormat, Result};

#[cfg(target_os = "linux")]
use crate::linux::V4l2LoopbackCamera;

#[derive(Clone, Debug)]
pub struct CameraBuilder {
    width: u32,
    height: u32,
    fps: f64,
    format: PixelFormat,
    devices: Option<Vec<String>>,
}

impl CameraBuilder {
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            format: PixelFormat::Rgb,
            devices: None,
        }
    }

    pub fn format(mut self, format: PixelFormat) -> Self {
        self.format = format;
        self
    }

    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.devices = Some(vec![device.into()]);
        self
    }

    pub fn devices(mut self, devices: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.devices = Some(devices.into_iter().map(Into::into).collect());
        self
    }

    pub fn build(self) -> Result<Camera> {
        let _fps = self.fps;

        #[cfg(target_os = "linux")]
        {
            let backend = V4l2LoopbackCamera::new(
                self.width,
                self.height,
                self.format.fourcc(),
                self.devices,
            )?;
            Ok(Camera::V4l2Loopback(backend))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::Error::runtime(
                "No Rust virtual camera backend is available on this platform yet.",
            ))
        }
    }
}

pub enum Camera {
    #[cfg(target_os = "linux")]
    V4l2Loopback(V4l2LoopbackCamera),
}

impl Camera {
    pub fn send(&mut self, frame: &[u8]) -> Result<()> {
        match self {
            #[cfg(target_os = "linux")]
            Self::V4l2Loopback(backend) => backend.send(frame),
        }
    }

    pub fn close(&mut self) {
        match self {
            #[cfg(target_os = "linux")]
            Self::V4l2Loopback(backend) => backend.stop(),
        }
    }

    pub fn device(&self) -> String {
        match self {
            #[cfg(target_os = "linux")]
            Self::V4l2Loopback(backend) => backend.device(),
        }
    }

    pub fn native_format(&self) -> PixelFormat {
        match self {
            #[cfg(target_os = "linux")]
            Self::V4l2Loopback(backend) => backend.native_format(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults_to_rgb_and_auto_device() {
        let builder = CameraBuilder::new(4, 2, 30.0);

        assert_eq!(builder.width, 4);
        assert_eq!(builder.height, 2);
        assert_eq!(builder.fps, 30.0);
        assert_eq!(builder.format, PixelFormat::Rgb);
        assert!(builder.devices.is_none());
    }

    #[test]
    fn builder_accepts_explicit_devices() {
        let builder = CameraBuilder::new(4, 2, 30.0)
            .format(PixelFormat::Bgr)
            .devices(["/dev/video0", "/dev/video1"]);

        assert_eq!(builder.format, PixelFormat::Bgr);
        assert_eq!(
            builder.devices.unwrap(),
            vec!["/dev/video0".to_owned(), "/dev/video1".to_owned()]
        );
    }
}
