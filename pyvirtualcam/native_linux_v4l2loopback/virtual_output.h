#pragma once

#include <errno.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/videodev2.h>

#include <string>
#include <vector>
#include <set>
#include <sstream>
#include <stdexcept>

#include "../native_shared/image_formats.h"

// v4l2loopback allows opening a device multiple times.
// To avoid selecting the same device more than once,
// we keep track of the ones we have open ourselves.
// Obviously, this won't help if multiple processes are used
// or if devices are opened by other tools.
// In this case, explicitly specifying the device seems the only solution.
static std::set<std::string> ACTIVE_DEVICES;

class VirtualOutput {
  private:
    bool _output_running = false;
    std::vector<int> _camera_fds;
    std::vector<std::string> _camera_devices;
    uint32_t _frame_fourcc;
    uint32_t _native_fourcc;
    uint32_t _frame_width;
    uint32_t _frame_height;
    uint32_t _out_frame_size;
    std::vector<uint8_t> _buffer_output;

  public:
    VirtualOutput(uint32_t width, uint32_t height, uint32_t fourcc,
                  std::optional<std::vector<std::string>> devices_) {
        _frame_width = width;
        _frame_height = height;
        _frame_fourcc = libyuv::CanonicalFourCC(fourcc);
        
        uint32_t out_frame_fmt_v4l;

        switch (_frame_fourcc) {
            case libyuv::FOURCC_RAW:
            case libyuv::FOURCC_24BG:
                // RGB|BGR -> I420
                _out_frame_size = i420_frame_size(width, height);
                _buffer_output.resize(_out_frame_size);
                _native_fourcc = libyuv::FOURCC_I420;
                out_frame_fmt_v4l = V4L2_PIX_FMT_YUV420;
                break;
            case libyuv::FOURCC_J400:
                _out_frame_size = gray_frame_size(width, height);
                _native_fourcc = _frame_fourcc;
                out_frame_fmt_v4l = V4L2_PIX_FMT_GREY;
                break;
            case libyuv::FOURCC_I420:
                _out_frame_size = i420_frame_size(width, height);
                _native_fourcc = _frame_fourcc;
                out_frame_fmt_v4l = V4L2_PIX_FMT_YUV420;
                break;
            case libyuv::FOURCC_NV12:
                _out_frame_size = nv12_frame_size(width, height);
                _native_fourcc = _frame_fourcc;
                out_frame_fmt_v4l = V4L2_PIX_FMT_NV12;
                break;
            case libyuv::FOURCC_YUY2:
                _out_frame_size = yuyv_frame_size(width, height);
                _native_fourcc = _frame_fourcc;
                out_frame_fmt_v4l = V4L2_PIX_FMT_YUYV;
                break;
            case libyuv::FOURCC_UYVY:
                _out_frame_size = uyvy_frame_size(width, height);
                _native_fourcc = _frame_fourcc;
                out_frame_fmt_v4l = V4L2_PIX_FMT_UYVY;
                break;
            default:
                throw std::runtime_error("Unsupported image format.");
        }

        auto try_open = [&](const std::string& device_name) -> int {
            if (ACTIVE_DEVICES.count(device_name)) {
                throw std::invalid_argument(
                    "Device " + device_name + " is already in use."
                );
            }
            int camera_fd = open(device_name.c_str(), O_WRONLY | O_SYNC);
            if (camera_fd == -1) {
                if (errno == EACCES) {
                    throw std::runtime_error(
                        "Could not access " + device_name + " due to missing permissions. "
                        "Did you add your user to the 'video' group? "
                        "Run 'usermod -a -G video myusername' and log out and in again."
                    );
                } else if (errno == ENOENT) {
                    throw std::invalid_argument(
                        "Device " + device_name + " does not exist."
                    );
                } else {
                    throw std::invalid_argument(
                        "Device " + device_name + " could not be opened: " +
                        std::string(strerror(errno))
                    );
                }
            }

            try {
                struct v4l2_capability camera_cap;

                if (ioctl(camera_fd, VIDIOC_QUERYCAP, &camera_cap) == -1) {
                    throw std::invalid_argument(
                        "Device capabilities of " + device_name + " could not be queried."
                    );
                }
                if (!(camera_cap.capabilities & V4L2_CAP_VIDEO_OUTPUT)) {
                    throw std::invalid_argument(
                        "Device " + device_name + " is not a video output device."
                    );
                }
                if (strcmp((const char*)camera_cap.driver, "v4l2 loopback") != 0) {
                    throw std::invalid_argument(
                        "Device " + device_name + " is not a V4L2 device."
                    );
                }
            } catch (std::exception &ex) {
                close(camera_fd);
                throw;
            }
            return camera_fd;
        };

        bool auto_detect = !devices_.has_value();
        std::vector<std::string> device_names;

        if (!auto_detect) {
            device_names = devices_.value();
            if (device_names.empty()) {
                throw std::invalid_argument("Device list cannot be empty.");
            }
        } else {
            // Auto-detect all potential v4l2loopback devices
            bool found = false;
            for (size_t i = 0; i < 100; i++) {
                std::ostringstream device_name_s;
                device_name_s << "/dev/video" << i;
                std::string device_name = device_name_s.str();

                // Check if device exists and is valid, but don't keep it open
                int test_fd = open(device_name.c_str(), O_WRONLY | O_SYNC);
                if (test_fd == -1) {
                    continue;
                }

                struct v4l2_capability camera_cap;
                bool is_valid = false;
                if (ioctl(test_fd, VIDIOC_QUERYCAP, &camera_cap) != -1 &&
                    (camera_cap.capabilities & V4L2_CAP_VIDEO_OUTPUT) &&
                    strcmp((const char*)camera_cap.driver, "v4l2 loopback") == 0) {
                    is_valid = true;
                }
                close(test_fd);

                if (is_valid) {
                    device_names.push_back(device_name);
                    found = true;
                }
            }
            if (!found) {
                throw std::runtime_error(
                    "No v4l2 loopback device found at /dev/video[0-99]. "
                    "Did you run 'modprobe v4l2loopback'? "
                    "See also pyvirtualcam's documentation.");
            }
        }

        auto cleanup_open_devices = [&]() {
            for (int fd : _camera_fds) {
                close(fd);
            }
            for (const auto& dev : _camera_devices) {
                ACTIVE_DEVICES.erase(dev);
            }
        };

        bool opened_device = false;

        // Open and configure all devices
        for (const auto& device_name : device_names) {
            int camera_fd;
            try {
                camera_fd = try_open(device_name);
            } catch (const std::invalid_argument& ex) {
                if (auto_detect) {
                    continue;
                }
                cleanup_open_devices();
                throw;
            } catch (std::exception& ex) {
                cleanup_open_devices();
                throw;
            }

            v4l2_format v4l2_fmt;
            memset(&v4l2_fmt, 0, sizeof(v4l2_fmt));
            v4l2_fmt.type = V4L2_BUF_TYPE_VIDEO_OUTPUT;
            v4l2_pix_format& pix = v4l2_fmt.fmt.pix;
            pix.width = width;
            pix.height = height;
            pix.pixelformat = out_frame_fmt_v4l;

            // v4l2loopback sets bytesperline, sizeimage, and colorspace for us.

            if (ioctl(camera_fd, VIDIOC_S_FMT, &v4l2_fmt) == -1) {
                close(camera_fd);
                // Close any already opened devices before throwing
                for (int fd : _camera_fds) {
                    close(fd);
                }
                for (const auto& dev : _camera_devices) {
                    ACTIVE_DEVICES.erase(dev);
                }
                throw std::runtime_error(
                    "Virtual camera device " + device_name +
                    " could not be configured: " + std::string(strerror(errno))
                );
            }

            _camera_fds.push_back(camera_fd);
            _camera_devices.push_back(device_name);
            ACTIVE_DEVICES.insert(device_name);
            opened_device = true;

            if (auto_detect) {
                break;
            }
        }

        if (!opened_device) {
            if (auto_detect) {
                throw std::runtime_error(
                    "All v4l2 loopback devices at /dev/video[0-99] are busy. "
                    "Is another process using them?"
                );
            }
            throw std::runtime_error("Failed to open any of the requested devices.");
        }

        _output_running = true;
    }

    void stop() {
        if (!_output_running) {
            return;
        }

        for (int fd : _camera_fds) {
            close(fd);
        }

        for (const auto& device : _camera_devices) {
            ACTIVE_DEVICES.erase(device);
        }

        _output_running = false;
    }

    void send(const uint8_t* frame) {
        if (!_output_running)
            return;

        uint8_t* out_frame;

        switch (_frame_fourcc) {
            case libyuv::FOURCC_RAW:
                out_frame = _buffer_output.data();
                rgb_to_i420(frame, out_frame, _frame_width, _frame_height);
                break;
            case libyuv::FOURCC_24BG:
                out_frame = _buffer_output.data();
                bgr_to_i420(frame, out_frame, _frame_width, _frame_height);
                break;
            case libyuv::FOURCC_J400:
            case libyuv::FOURCC_I420:
            case libyuv::FOURCC_NV12:
            case libyuv::FOURCC_YUY2:
            case libyuv::FOURCC_UYVY:
                out_frame = const_cast<uint8_t*>(frame);
                break;
            default:
                throw std::logic_error("not implemented");
        }

        // Write to all devices
        for (size_t i = 0; i < _camera_fds.size(); i++) {
            ssize_t n = write(_camera_fds[i], out_frame, _out_frame_size);
            if (n == -1) {
                // not an exception, in case it is temporary
                fprintf(stderr, "error writing frame to %s: %s\n",
                        _camera_devices[i].c_str(), strerror(errno));
            }
        }
    }

    std::string device() {
        if (_camera_devices.empty()) {
            return "";
        }
        std::string result = _camera_devices[0];
        for (size_t i = 1; i < _camera_devices.size(); i++) {
            result += ", " + _camera_devices[i];
        }
        return result;
    }

    uint32_t native_fourcc() {
        return _native_fourcc;
    }
};
