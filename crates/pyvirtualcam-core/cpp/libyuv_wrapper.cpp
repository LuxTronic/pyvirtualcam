#include <cmath>
#include <cstdint>

#include <libyuv.h>

extern "C" uint32_t pyvc_canonical_fourcc(uint32_t fourcc) {
    return libyuv::CanonicalFourCC(fourcc);
}

extern "C" void pyvc_rgb_to_i420(
    const uint8_t* rgb,
    uint8_t* i420,
    int32_t width,
    int32_t height
) {
    int32_t height_abs = std::abs(height);
    int32_t chroma_width = (width + 1) / 2;
    int32_t chroma_height = (height_abs + 1) / 2;

    libyuv::RAWToI420(
        rgb, width * 3,
        i420, width,
        i420 + width * height_abs, chroma_width,
        i420 + width * height_abs + chroma_width * chroma_height, chroma_width,
        width, height);
}

extern "C" void pyvc_bgr_to_i420(
    const uint8_t* bgr,
    uint8_t* i420,
    int32_t width,
    int32_t height
) {
    int32_t height_abs = std::abs(height);
    int32_t chroma_width = (width + 1) / 2;
    int32_t chroma_height = (height_abs + 1) / 2;

    libyuv::RGB24ToI420(
        bgr, width * 3,
        i420, width,
        i420 + width * height_abs, chroma_width,
        i420 + width * height_abs + chroma_width * chroma_height, chroma_width,
        width, height);
}
