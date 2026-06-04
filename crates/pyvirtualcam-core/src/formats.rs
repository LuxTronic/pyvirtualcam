use crate::error::{Error, Result};
use crate::fourcc::{
    FourCc, FOURCC_24BG, FOURCC_ABGR, FOURCC_I420, FOURCC_J400, FOURCC_NV12, FOURCC_RAW,
    FOURCC_UYVY, FOURCC_YUY2,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    Rgba,
    Gray,
    I420,
    Nv12,
    Yuyv,
    Uyvy,
}

impl PixelFormat {
    pub fn from_canonical_fourcc(fourcc: FourCc) -> Result<Self> {
        match fourcc {
            FOURCC_RAW => Ok(Self::Rgb),
            FOURCC_24BG => Ok(Self::Bgr),
            FOURCC_ABGR => Ok(Self::Rgba),
            FOURCC_J400 => Ok(Self::Gray),
            FOURCC_I420 => Ok(Self::I420),
            FOURCC_NV12 => Ok(Self::Nv12),
            FOURCC_YUY2 => Ok(Self::Yuyv),
            FOURCC_UYVY => Ok(Self::Uyvy),
            _ => Err(Error::runtime("Unsupported image format.")),
        }
    }

    pub fn fourcc(self) -> FourCc {
        match self {
            Self::Rgb => FOURCC_RAW,
            Self::Bgr => FOURCC_24BG,
            Self::Rgba => FOURCC_ABGR,
            Self::Gray => FOURCC_J400,
            Self::I420 => FOURCC_I420,
            Self::Nv12 => FOURCC_NV12,
            Self::Yuyv => FOURCC_YUY2,
            Self::Uyvy => FOURCC_UYVY,
        }
    }

    pub fn frame_size(self, width: u32, height: u32) -> usize {
        let pixels = width as usize * height as usize;
        match self {
            Self::Rgb | Self::Bgr => pixels * 3,
            Self::Rgba => pixels * 4,
            Self::Gray => pixels,
            Self::I420 | Self::Nv12 => pixels * 3 / 2,
            Self::Yuyv | Self::Uyvy => pixels * 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_frame_sizes() {
        assert_eq!(PixelFormat::Rgb.frame_size(4, 2), 24);
        assert_eq!(PixelFormat::Bgr.frame_size(4, 2), 24);
        assert_eq!(PixelFormat::Rgba.frame_size(4, 2), 32);
        assert_eq!(PixelFormat::Gray.frame_size(4, 2), 8);
        assert_eq!(PixelFormat::I420.frame_size(4, 2), 12);
        assert_eq!(PixelFormat::Nv12.frame_size(4, 2), 12);
        assert_eq!(PixelFormat::Yuyv.frame_size(4, 2), 16);
        assert_eq!(PixelFormat::Uyvy.frame_size(4, 2), 16);
    }

    #[test]
    fn maps_supported_fourcc_values() {
        let formats = [
            PixelFormat::Rgb,
            PixelFormat::Bgr,
            PixelFormat::Rgba,
            PixelFormat::Gray,
            PixelFormat::I420,
            PixelFormat::Nv12,
            PixelFormat::Yuyv,
            PixelFormat::Uyvy,
        ];

        for format in formats {
            assert_eq!(
                PixelFormat::from_canonical_fourcc(format.fourcc()).unwrap(),
                format
            );
        }
    }
}
