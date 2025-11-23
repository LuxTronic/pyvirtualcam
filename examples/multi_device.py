"""
Example demonstrating frame duplication to multiple v4l2loopback devices.

This example shows how to send frames to multiple /dev/videoX devices
simultaneously with a single Camera instance. This is useful when you want
multiple applications to consume the same video stream without creating
multiple Camera instances (which would use more memory for duplicate
format conversions).

Requirements:
- Multiple v4l2loopback devices must be created first:
  sudo modprobe v4l2loopback devices=3
  This creates /dev/video0, /dev/video1, /dev/video2
"""

import colorsys
import numpy as np
import pyvirtualcam

# Single device (backward compatible)
print("Example 1: Single device")
with pyvirtualcam.Camera(width=1280, height=720, fps=20, device="/dev/video0") as cam:
    print(f'Using virtual camera: {cam.device}')
    frame = np.zeros((cam.height, cam.width, 3), np.uint8)
    for i in range(60):  # 3 seconds at 20 fps
        h, s, v = (i % 100) / 100, 1.0, 1.0
        r, g, b = colorsys.hsv_to_rgb(h, s, v)
        frame[:] = (r * 255, g * 255, b * 255)
        cam.send(frame)
        cam.sleep_until_next_frame()

# Multiple devices - frames duplicated to all with single conversion
print("\nExample 2: Multiple devices (frame duplication)")
with pyvirtualcam.Camera(width=1280, height=720, fps=20,
                         device=["/dev/video0", "/dev/video1", "/dev/video2"]) as cam:
    print(f'Using virtual cameras: {cam.device}')
    print('Note: Single RGB->I420 conversion, written to all 3 devices!')
    frame = np.zeros((cam.height, cam.width, 3), np.uint8)
    for i in range(60):  # 3 seconds at 20 fps
        h, s, v = (i % 100) / 100, 1.0, 1.0
        r, g, b = colorsys.hsv_to_rgb(h, s, v)
        frame[:] = (r * 255, g * 255, b * 255)
        cam.send(frame)
        cam.sleep_until_next_frame()

print("\nDone! The same frame was sent to all devices with minimal memory overhead.")
