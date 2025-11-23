#include <stdexcept>
#include <optional>
#include <pybind11/pybind11.h>
#include <pybind11/pytypes.h>
#include <pybind11/stl.h>
#include <pybind11/numpy.h>
#include "virtual_output.h"

namespace py = pybind11;

class Camera {
  private:
    VirtualOutput virtual_output;

    static std::string to_string_like(const py::handle& obj) {
        // Use py::str so any object with __str__ works (e.g., pathlib.Path)
        return py::str(obj).cast<std::string>();
    }

    static std::optional<std::vector<std::string>>
    parse_devices(const py::object& device_obj) {
        if (device_obj.is_none()) {
            return std::nullopt;
        }

        if (py::isinstance<py::sequence>(device_obj) &&
            !py::isinstance<py::str>(device_obj)) {
            py::list seq = py::list(device_obj);
            std::vector<std::string> devices;
            devices.reserve(seq.size());
            for (py::handle item : seq) {
                try {
                    devices.push_back(to_string_like(item));
                } catch (const py::error_already_set&) {
                    throw std::invalid_argument(
                        "Each device must be string-convertible when specifying a list of devices."
                    );
                }
            }
            return devices;
        }

        try {
            return std::vector<std::string>{to_string_like(device_obj)};
        } catch (const py::error_already_set&) {
            throw std::invalid_argument(
                "Device must be None, a string, or a sequence of strings."
            );
        }
    }

  public:
    Camera(uint32_t width, uint32_t height, [[maybe_unused]] double fps,
           uint32_t fourcc, py::object device_arg)
     : virtual_output {width, height, fourcc, parse_devices(device_arg)} {
    }

    void close() {
        virtual_output.stop();
    }

    std::string device() {
        return virtual_output.device();
    }

    uint32_t native_fourcc() {
        return virtual_output.native_fourcc();
    }

    void send(py::array_t<uint8_t, py::array::c_style> frame) {
        py::buffer_info buf = frame.request();    
        virtual_output.send(static_cast<uint8_t*>(buf.ptr));
    }
};

PYBIND11_MODULE(_native_linux_v4l2loopback, m) {
    py::class_<Camera>(m, "Camera")
        .def(py::init<uint32_t, uint32_t, double, uint32_t, py::object>(),
             py::kw_only(),
             py::arg("width"), py::arg("height"), py::arg("fps"),
             py::arg("fourcc"), py::arg("device") = py::none())
        .def("close", &Camera::close)
        .def("send", &Camera::send)
        .def("device", &Camera::device)
        .def("native_fourcc", &Camera::native_fourcc);
}
