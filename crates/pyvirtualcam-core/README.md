# pyvirtualcam-core

`pyvirtualcam-core` is the Rust implementation layer for pyvirtualcam.
It is designed to be used in two ways:

- as the backend for the existing Python package through the `pyvirtualcam-py`
  PyO3 extension crate
- as the foundation for a native Rust API

The Python package remains the compatibility surface for existing users. The
Rust core owns platform backend logic, pixel format handling, and conversion
wrappers as those pieces are migrated.

## Architecture

The migration is intentionally incremental:

```text
Python users
    -> pyvirtualcam.Camera
        -> pyvirtualcam-py PyO3 extension
            -> pyvirtualcam-core
                -> platform backend

Rust users
    -> pyvirtualcam-core CameraBuilder
        -> platform backend
```

Linux `v4l2loopback` is the first Rust backend. macOS and Windows still use the
existing native implementations until they can be ported and tested without
changing Python behavior.

## Pixel Conversion

This crate continues to build the vendored `external/libyuv` source instead of
reimplementing pixel conversion in Rust.

That choice is deliberate:

- existing pyvirtualcam behavior already depends on libyuv conversion details
- capture tests compare pixel output with tight tolerances
- using the same vendored source avoids system-library drift across wheel builds

The Rust code talks to libyuv through a small C ABI wrapper in
`cpp/libyuv_wrapper.cpp`. More conversions can be added there as additional
platform backends move into Rust.

## V4L2 ABI

The Linux backend uses a small hand-written subset of the V4L2 ABI rather than
bindgen-generated bindings. This avoids adding a libclang dependency to wheel
builds and keeps the reviewed surface small.

The ABI definitions are intentionally limited to the calls used by this backend:

- `VIDIOC_QUERYCAP`
- `VIDIOC_S_FMT`
- `struct v4l2_capability`
- `struct v4l2_format`
- `struct v4l2_pix_format`

Unit tests assert the relevant struct sizes and ioctl request numbers against
Linux header values. If the backend grows beyond this small surface, generated
bindings or a maintained V4L2 crate should be reconsidered.
