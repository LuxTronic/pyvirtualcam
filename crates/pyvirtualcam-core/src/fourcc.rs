pub type FourCc = u32;

pub const fn encode_fourcc(bytes: &[u8; 4]) -> FourCc {
    bytes[0] as FourCc
        | ((bytes[1] as FourCc) << 8)
        | ((bytes[2] as FourCc) << 16)
        | ((bytes[3] as FourCc) << 24)
}

pub const FOURCC_RAW: FourCc = encode_fourcc(b"raw ");
pub const FOURCC_24BG: FourCc = encode_fourcc(b"24BG");
pub const FOURCC_ABGR: FourCc = encode_fourcc(b"ABGR");
pub const FOURCC_J400: FourCc = encode_fourcc(b"J400");
pub const FOURCC_I420: FourCc = encode_fourcc(b"I420");
pub const FOURCC_NV12: FourCc = encode_fourcc(b"NV12");
pub const FOURCC_YUY2: FourCc = encode_fourcc(b"YUY2");
pub const FOURCC_UYVY: FourCc = encode_fourcc(b"UYVY");

pub const V4L2_PIX_FMT_GREY: FourCc = encode_fourcc(b"GREY");
pub const V4L2_PIX_FMT_NV12: FourCc = FOURCC_NV12;
pub const V4L2_PIX_FMT_UYVY: FourCc = FOURCC_UYVY;
pub const V4L2_PIX_FMT_YUYV: FourCc = encode_fourcc(b"YUYV");
pub const V4L2_PIX_FMT_YUV420: FourCc = encode_fourcc(b"YU12");

pub fn canonical_fourcc(fourcc: FourCc) -> FourCc {
    crate::convert::canonical_fourcc(fourcc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_fourcc_like_python() {
        assert_eq!(encode_fourcc(b"I420"), 0x30323449);
        assert_eq!(encode_fourcc(b"raw "), 0x20776172);
    }
}
