import pytest
import numpy as np

import pyvirtualcam
from pyvirtualcam import PixelFormat
from pyvirtualcam.util import encode_fourcc


class MockBackend:
    instances = []

    def __init__(self, *, width, height, fps, fourcc, device, **kw):
        self.width = width
        self.height = height
        self.fps = fps
        self.fourcc = fourcc
        self.device_arg = device
        self.kw = kw
        self.closed = False
        self.frames = []
        MockBackend.instances.append(self)

    def close(self):
        self.closed = True

    def send(self, frame):
        self.frames.append(frame.copy())

    def device(self):
        return "mock-device"

    def native_fourcc(self):
        return self.fourcc


@pytest.fixture
def mock_backend():
    previous = pyvirtualcam.camera.BACKENDS.get("mock")
    MockBackend.instances = []
    pyvirtualcam.register_backend("mock", MockBackend)
    try:
        yield MockBackend
    finally:
        if previous is None:
            pyvirtualcam.camera.BACKENDS.pop("mock", None)
        else:
            pyvirtualcam.register_backend("mock", previous)


def test_backend_constructor_contract(mock_backend):
    with pyvirtualcam.Camera(
        width=4,
        height=2,
        fps=30,
        fmt=PixelFormat.BGR,
        device="custom-device",
        backend="mock",
        custom=True,
    ) as cam:
        backend = mock_backend.instances[-1]

        assert cam.backend == "mock"
        assert cam.device == "mock-device"
        assert cam.width == 4
        assert cam.height == 2
        assert cam.fps == 30
        assert cam.fmt == PixelFormat.BGR
        assert cam.native_fmt == PixelFormat.BGR
        assert backend.device_arg == "custom-device"
        assert backend.kw == {"custom": True}
        assert backend.fourcc == encode_fourcc(PixelFormat.BGR.value)

    assert backend.closed


def test_send_validates_and_flattens_frame(mock_backend):
    with pyvirtualcam.Camera(4, 2, 30, backend="mock") as cam:
        frame = np.arange(4 * 2 * 3, dtype=np.uint8).reshape(2, 4, 3)

        cam.send(frame)

        backend = mock_backend.instances[-1]
        assert cam.frames_sent == 1
        assert len(backend.frames) == 1
        assert backend.frames[0].shape == (4 * 2 * 3,)
        assert np.array_equal(backend.frames[0], frame.reshape(-1))


def test_send_rejects_wrong_dtype_and_shape(mock_backend):
    with pyvirtualcam.Camera(4, 2, 30, backend="mock") as cam:
        with pytest.raises(TypeError):
            cam.send(np.zeros((2, 4, 3), dtype=np.uint16))

        with pytest.raises(ValueError):
            cam.send(np.zeros((4, 2, 3), dtype=np.uint8))


def test_close_is_idempotent(mock_backend):
    cam = pyvirtualcam.Camera(4, 2, 30, backend="mock")
    backend = mock_backend.instances[-1]

    cam.close()
    cam.close()

    assert backend.closed
