//! Linux `v4l2loopback` backend.
//!
//! This module uses a deliberately small hand-written V4L2 ABI surface. The
//! project still builds binary wheels, so avoiding bindgen keeps libclang out of
//! the build requirements. The definitions below are covered by tests for the
//! struct sizes and ioctl request numbers used by this backend.

use std::collections::HashSet;
use std::mem;
use std::os::fd::RawFd;
use std::sync::{Mutex, OnceLock};

use crate::convert;
use crate::error::{Error, Result};
use crate::formats::PixelFormat;
use crate::fourcc::{
    canonical_fourcc, FourCc, V4L2_PIX_FMT_GREY, V4L2_PIX_FMT_NV12, V4L2_PIX_FMT_UYVY,
    V4L2_PIX_FMT_YUV420, V4L2_PIX_FMT_YUYV,
};

const V4L2_CAP_VIDEO_OUTPUT: u32 = 0x0000_0002;
const V4L2_BUF_TYPE_VIDEO_OUTPUT: u32 = 2;
const V4L2_LOOPBACK_DRIVER: &[u8] = b"v4l2 loopback";
const IOC_WRITE: u32 = 1;
const IOC_READ: u32 = 2;

static ACTIVE_DEVICES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn active_devices() -> &'static Mutex<HashSet<String>> {
    ACTIVE_DEVICES.get_or_init(|| Mutex::new(HashSet::new()))
}

pub struct V4l2LoopbackCamera {
    running: bool,
    camera_fds: Vec<RawFd>,
    camera_devices: Vec<String>,
    frame_format: PixelFormat,
    native_format: PixelFormat,
    frame_width: u32,
    frame_height: u32,
    out_frame_size: usize,
    buffer_output: Vec<u8>,
}

impl V4l2LoopbackCamera {
    pub fn new(
        width: u32,
        height: u32,
        fourcc: FourCc,
        devices: Option<Vec<String>>,
    ) -> Result<Self> {
        let frame_fourcc = canonical_fourcc(fourcc);
        let frame_format = PixelFormat::from_canonical_fourcc(frame_fourcc)?;

        let (native_format, out_frame_fmt_v4l) = match frame_format {
            PixelFormat::Rgb | PixelFormat::Bgr => (PixelFormat::I420, V4L2_PIX_FMT_YUV420),
            PixelFormat::Gray => (PixelFormat::Gray, V4L2_PIX_FMT_GREY),
            PixelFormat::I420 => (PixelFormat::I420, V4L2_PIX_FMT_YUV420),
            PixelFormat::Nv12 => (PixelFormat::Nv12, V4L2_PIX_FMT_NV12),
            PixelFormat::Yuyv => (PixelFormat::Yuyv, V4L2_PIX_FMT_YUYV),
            PixelFormat::Uyvy => (PixelFormat::Uyvy, V4L2_PIX_FMT_UYVY),
            PixelFormat::Rgba => return Err(Error::runtime("Unsupported image format.")),
        };

        let out_frame_size = native_format.frame_size(width, height);
        let mut buffer_output = Vec::new();
        if matches!(frame_format, PixelFormat::Rgb | PixelFormat::Bgr) {
            buffer_output.resize(out_frame_size, 0);
        }

        let auto_detect = devices.is_none();
        let device_names = if let Some(devices) = devices {
            if devices.is_empty() {
                return Err(Error::invalid_argument("Device list cannot be empty."));
            }
            devices
        } else {
            discover_devices()?
        };

        let mut camera_fds = Vec::new();
        let mut camera_devices = Vec::new();
        let mut opened_device = false;

        for device_name in device_names {
            let camera_fd = match try_open(&device_name) {
                Ok(fd) => fd,
                Err(err) if auto_detect && matches!(err, Error::InvalidArgument(_)) => continue,
                Err(err) => {
                    cleanup_open_devices(&camera_fds, &camera_devices);
                    return Err(err);
                }
            };

            if let Err(err) =
                configure_device(camera_fd, &device_name, width, height, out_frame_fmt_v4l)
            {
                unsafe {
                    libc::close(camera_fd);
                }
                release_device(&device_name);
                cleanup_open_devices(&camera_fds, &camera_devices);
                return Err(err);
            }

            camera_fds.push(camera_fd);
            camera_devices.push(device_name);
            opened_device = true;

            if auto_detect {
                break;
            }
        }

        if !opened_device {
            if auto_detect {
                return Err(Error::runtime(
                    "All v4l2 loopback devices at /dev/video[0-99] are busy. Is another process using them?",
                ));
            }
            return Err(Error::runtime(
                "Failed to open any of the requested devices.",
            ));
        }

        Ok(Self {
            running: true,
            camera_fds,
            camera_devices,
            frame_format,
            native_format,
            frame_width: width,
            frame_height: height,
            out_frame_size,
            buffer_output,
        })
    }

    pub fn stop(&mut self) {
        if !self.running {
            return;
        }

        cleanup_open_devices(&self.camera_fds, &self.camera_devices);
        self.running = false;
    }

    pub fn send(&mut self, frame: &[u8]) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        let expected_frame_size = self
            .frame_format
            .frame_size(self.frame_width, self.frame_height);
        if frame.len() < expected_frame_size {
            return Err(Error::invalid_argument(
                "input frame is smaller than expected",
            ));
        }

        let out_frame = match self.frame_format {
            PixelFormat::Rgb => {
                convert::rgb_to_i420(
                    frame,
                    &mut self.buffer_output,
                    self.frame_width,
                    self.frame_height,
                )?;
                self.buffer_output.as_slice()
            }
            PixelFormat::Bgr => {
                convert::bgr_to_i420(
                    frame,
                    &mut self.buffer_output,
                    self.frame_width,
                    self.frame_height,
                )?;
                self.buffer_output.as_slice()
            }
            PixelFormat::Gray
            | PixelFormat::I420
            | PixelFormat::Nv12
            | PixelFormat::Yuyv
            | PixelFormat::Uyvy => frame,
            PixelFormat::Rgba => return Err(Error::runtime("Unsupported image format.")),
        };

        for (fd, device) in self.camera_fds.iter().zip(&self.camera_devices) {
            let written =
                unsafe { libc::write(*fd, out_frame.as_ptr().cast(), self.out_frame_size) };
            if written == -1 {
                eprintln!(
                    "error writing frame to {}: {}",
                    device,
                    std::io::Error::last_os_error()
                );
            }
        }

        Ok(())
    }

    pub fn device(&self) -> String {
        self.camera_devices.join(", ")
    }

    pub fn native_fourcc(&self) -> FourCc {
        self.native_format.fourcc()
    }

    pub fn native_format(&self) -> PixelFormat {
        self.native_format
    }
}

impl Drop for V4l2LoopbackCamera {
    fn drop(&mut self) {
        self.stop();
    }
}

fn discover_devices() -> Result<Vec<String>> {
    let mut device_names = Vec::new();

    for i in 0..100 {
        let device_name = format!("/dev/video{i}");
        let c_device_name = c_string(&device_name)?;
        let fd = unsafe { libc::open(c_device_name.as_ptr(), libc::O_WRONLY | libc::O_SYNC) };
        if fd == -1 {
            continue;
        }

        let is_valid = validate_fd(fd, &device_name).is_ok();
        unsafe {
            libc::close(fd);
        }

        if is_valid {
            device_names.push(device_name);
        }
    }

    if device_names.is_empty() {
        return Err(Error::runtime(
            "No v4l2 loopback device found at /dev/video[0-99]. Did you run 'modprobe v4l2loopback'? See also pyvirtualcam's documentation.",
        ));
    }

    Ok(device_names)
}

fn try_open(device_name: &str) -> Result<RawFd> {
    {
        let mut active = active_devices().lock().unwrap();
        if active.contains(device_name) {
            return Err(Error::invalid_argument(format!(
                "Device {device_name} is already in use."
            )));
        }
        active.insert(device_name.to_string());
    }

    let release_on_error = |device_name: &str| {
        release_device(device_name);
    };

    let c_device_name = match c_string(device_name) {
        Ok(value) => value,
        Err(err) => {
            release_on_error(device_name);
            return Err(err);
        }
    };
    let camera_fd = unsafe { libc::open(c_device_name.as_ptr(), libc::O_WRONLY | libc::O_SYNC) };
    if camera_fd == -1 {
        release_on_error(device_name);
        let err = std::io::Error::last_os_error();
        match err.raw_os_error() {
            Some(libc::EACCES) => {
                return Err(Error::runtime(format!(
                    "Could not access {device_name} due to missing permissions. Did you add your user to the 'video' group? Run 'usermod -a -G video myusername' and log out and in again."
                )));
            }
            Some(libc::ENOENT) => {
                return Err(Error::invalid_argument(format!(
                    "Device {device_name} does not exist."
                )));
            }
            _ => {
                return Err(Error::invalid_argument(format!(
                    "Device {device_name} could not be opened: {err}"
                )));
            }
        }
    }

    if let Err(err) = validate_fd(camera_fd, device_name) {
        unsafe {
            libc::close(camera_fd);
        }
        release_on_error(device_name);
        return Err(err);
    }

    Ok(camera_fd)
}

fn release_device(device_name: &str) {
    active_devices().lock().unwrap().remove(device_name);
}

fn validate_fd(camera_fd: RawFd, device_name: &str) -> Result<()> {
    let mut capability = V4l2Capability::default();
    let result = unsafe { libc::ioctl(camera_fd, vidioc_querycap(), &mut capability) };
    if result == -1 {
        return Err(Error::invalid_argument(format!(
            "Device capabilities of {device_name} could not be queried."
        )));
    }

    if capability.capabilities & V4L2_CAP_VIDEO_OUTPUT == 0 {
        return Err(Error::invalid_argument(format!(
            "Device {device_name} is not a video output device."
        )));
    }

    let driver = nul_terminated_bytes(&capability.driver);
    if driver != V4L2_LOOPBACK_DRIVER {
        return Err(Error::invalid_argument(format!(
            "Device {device_name} is not a V4L2 device."
        )));
    }

    Ok(())
}

fn configure_device(
    camera_fd: RawFd,
    device_name: &str,
    width: u32,
    height: u32,
    pixelformat: FourCc,
) -> Result<()> {
    let pix = V4l2PixFormat {
        width,
        height,
        pixelformat,
        ..Default::default()
    };
    let mut format = V4l2Format {
        type_: V4L2_BUF_TYPE_VIDEO_OUTPUT,
        fmt: V4l2FormatUnion { pix },
    };

    let result = unsafe { libc::ioctl(camera_fd, vidioc_s_fmt(), &mut format) };
    if result == -1 {
        return Err(Error::runtime(format!(
            "Virtual camera device {device_name} could not be configured: {}",
            std::io::Error::last_os_error()
        )));
    }

    Ok(())
}

fn cleanup_open_devices(camera_fds: &[RawFd], camera_devices: &[String]) {
    for fd in camera_fds {
        unsafe {
            libc::close(*fd);
        }
    }

    let mut active = active_devices().lock().unwrap();
    for device in camera_devices {
        active.remove(device);
    }
}

fn c_string(value: &str) -> Result<std::ffi::CString> {
    std::ffi::CString::new(value)
        .map_err(|_| Error::invalid_argument("device name must not contain NUL bytes"))
}

fn nul_terminated_bytes(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

fn vidioc_querycap() -> libc::c_ulong {
    ior::<V4l2Capability>(b'V', 0)
}

fn vidioc_s_fmt() -> libc::c_ulong {
    iowr::<V4l2Format>(b'V', 5)
}

fn ior<T>(type_: u8, nr: u8) -> libc::c_ulong {
    ioc(IOC_READ, type_, nr, mem::size_of::<T>())
}

fn iowr<T>(type_: u8, nr: u8) -> libc::c_ulong {
    ioc(IOC_READ | IOC_WRITE, type_, nr, mem::size_of::<T>())
}

fn ioc(dir: u32, type_: u8, nr: u8, size: usize) -> libc::c_ulong {
    const IOC_NRBITS: u32 = 8;
    const IOC_TYPEBITS: u32 = 8;
    const IOC_SIZEBITS: u32 = 14;

    const IOC_NRSHIFT: u32 = 0;
    const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
    const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
    const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;

    ((dir as libc::c_ulong) << IOC_DIRSHIFT)
        | ((type_ as libc::c_ulong) << IOC_TYPESHIFT)
        | ((nr as libc::c_ulong) << IOC_NRSHIFT)
        | ((size as libc::c_ulong) << IOC_SIZESHIFT)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct V4l2Capability {
    driver: [u8; 16],
    card: [u8; 32],
    bus_info: [u8; 32],
    version: u32,
    capabilities: u32,
    device_caps: u32,
    reserved: [u32; 3],
}

// Minimal mirror of `struct v4l2_pix_format` from linux/videodev2.h.
// Keep this private and tested; if more V4L2 APIs are needed, prefer moving to
// generated or crate-backed bindings instead of expanding this by hand.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct V4l2PixFormat {
    width: u32,
    height: u32,
    pixelformat: u32,
    field: u32,
    bytesperline: u32,
    sizeimage: u32,
    colorspace: u32,
    priv_: u32,
    flags: u32,
    ycbcr_enc: u32,
    quantization: u32,
    xfer_func: u32,
}

// `struct v4l2_format` contains a large union. Only `pix` is used here, but the
// raw member must preserve the full Linux ABI size so ioctl request numbers
// match the platform headers.
#[repr(C)]
union V4l2FormatUnion {
    pix: V4l2PixFormat,
    raw_data: [u8; 204],
}

#[repr(C)]
struct V4l2Format {
    type_: u32,
    fmt: V4l2FormatUnion,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn v4l2_struct_layout_matches_linux_headers() {
        assert_eq!(mem::size_of::<V4l2Capability>(), 104);
        assert_eq!(mem::size_of::<V4l2PixFormat>(), 48);
        assert_eq!(mem::size_of::<V4l2Format>(), 208);
        assert_eq!(offset_of!(V4l2Format, fmt), 4);
        assert_eq!(offset_of!(V4l2PixFormat, pixelformat), 8);
    }

    #[test]
    fn v4l2_ioctl_numbers_match_linux_headers() {
        assert_eq!(vidioc_querycap(), 0x8068_5600);
        assert_eq!(vidioc_s_fmt(), 0xc0d0_5605);
    }

    #[test]
    fn trims_nul_terminated_driver_names() {
        let mut driver = [0; 16];
        driver[..V4L2_LOOPBACK_DRIVER.len()].copy_from_slice(V4L2_LOOPBACK_DRIVER);

        assert_eq!(nul_terminated_bytes(&driver), V4L2_LOOPBACK_DRIVER);
    }
}
