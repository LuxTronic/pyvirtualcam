API Reference
=============

Implementation Notes
--------------------

The Linux ``v4l2loopback`` backend is implemented in Rust and exposed through
the same Python API documented below. macOS and Windows continue to use the
existing native backends.

The Rust core crate also includes an initial native Rust API for Linux users.
See ``crates/pyvirtualcam-core/README.md`` and
``crates/pyvirtualcam-core/examples/simple.rs`` in the source tree.

.. autoclass:: pyvirtualcam.Camera
   :members:
   :member-order: groupwise

.. autoclass:: pyvirtualcam.PixelFormat
   :members:
   :member-order: groupwise

.. autofunction:: pyvirtualcam.register_backend

.. autoclass:: pyvirtualcam.Backend
   :members:
   :member-order: groupwise