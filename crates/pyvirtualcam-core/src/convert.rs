use crate::error::{Error, Result};
use crate::fourcc::FourCc;
use crate::PixelFormat;

extern "C" {
    fn pyvc_canonical_fourcc(fourcc: FourCc) -> FourCc;
    fn pyvc_rgb_to_i420(rgb: *const u8, i420: *mut u8, width: i32, height: i32);
    fn pyvc_bgr_to_i420(bgr: *const u8, i420: *mut u8, width: i32, height: i32);
}

pub fn canonical_fourcc(fourcc: FourCc) -> FourCc {
    unsafe { pyvc_canonical_fourcc(fourcc) }
}

pub fn rgb_to_i420(input: &[u8], output: &mut [u8], width: u32, height: u32) -> Result<()> {
    convert_to_i420(PixelFormat::Rgb, input, output, width, height)
}

pub fn bgr_to_i420(input: &[u8], output: &mut [u8], width: u32, height: u32) -> Result<()> {
    convert_to_i420(PixelFormat::Bgr, input, output, width, height)
}

fn convert_to_i420(
    input_format: PixelFormat,
    input: &[u8],
    output: &mut [u8],
    width: u32,
    height: u32,
) -> Result<()> {
    let expected_input = input_format.frame_size(width, height);
    let expected_output = PixelFormat::I420.frame_size(width, height);
    if input.len() < expected_input {
        return Err(Error::invalid_argument(
            "input frame is smaller than expected",
        ));
    }
    if output.len() < expected_output {
        return Err(Error::invalid_argument(
            "output frame is smaller than expected",
        ));
    }

    let width = width as i32;
    let height = height as i32;

    unsafe {
        match input_format {
            PixelFormat::Rgb => {
                pyvc_rgb_to_i420(input.as_ptr(), output.as_mut_ptr(), width, height);
            }
            PixelFormat::Bgr => {
                pyvc_bgr_to_i420(input.as_ptr(), output.as_mut_ptr(), width, height);
            }
            _ => unreachable!("only RGB and BGR convert to I420 here"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_input_buffer() {
        let mut output = vec![0; PixelFormat::I420.frame_size(4, 2)];

        let err = rgb_to_i420(&[0; 1], &mut output, 4, 2).unwrap_err();

        assert!(matches!(err, Error::InvalidArgument(_)));
    }

    #[test]
    fn rejects_short_output_buffer() {
        let input = vec![0; PixelFormat::Rgb.frame_size(4, 2)];

        let err = rgb_to_i420(&input, &mut [0; 1], 4, 2).unwrap_err();

        assert!(matches!(err, Error::InvalidArgument(_)));
    }

    #[test]
    fn converts_rgb_to_i420_with_expected_size() {
        let input = vec![128; PixelFormat::Rgb.frame_size(4, 2)];
        let mut output = vec![0; PixelFormat::I420.frame_size(4, 2)];

        rgb_to_i420(&input, &mut output, 4, 2).unwrap();

        assert_eq!(output.len(), 12);
    }
}
