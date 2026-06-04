# Agent Guidance

This repository provides `pyvirtualcam`, a Python package with native virtual
camera backends. Keep the public Python API stable unless a change is explicitly
intended to be breaking.

## Architecture

- `pyvirtualcam/` contains the Python compatibility API.
- `crates/pyvirtualcam-core/` contains the Rust core and the Linux
  `v4l2loopback` backend.
- `crates/pyvirtualcam-py/` contains the PyO3 module exported as
  `pyvirtualcam._native_linux_v4l2loopback`.
- macOS and Windows still use the existing native C++/ObjC++ backends.

## Development Notes

- Preserve `pyvirtualcam.Camera`, `PixelFormat`, `Backend`, and
  `register_backend` behavior.
- Keep pixel conversions backed by vendored `external/libyuv` unless parity is
  proven with capture tests.
- Treat the hand-written V4L2 ABI definitions as a small, private surface. If
  the Linux backend needs more V4L2 APIs, prefer generated or crate-backed
  bindings.
- Run Rust checks with `cargo fmt --check`, `cargo clippy --workspace --locked
  --all-targets -- -D warnings`, and `cargo test --workspace --locked`.
- Run Python checks with `python -m pytest -q test/test_backend_contract.py
  test/test_util.py`; full capture tests need real virtual camera devices.
