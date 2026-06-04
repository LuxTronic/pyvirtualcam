use std::time::Duration;

use pyvirtualcam_core::{CameraBuilder, PixelFormat};

fn main() -> pyvirtualcam_core::Result<()> {
    let width = 640;
    let height = 480;
    let fps = 20.0;
    let mut camera = CameraBuilder::new(width, height, fps)
        .format(PixelFormat::Rgb)
        .build()?;

    println!(
        "Using virtual camera: {} ({:?})",
        camera.device(),
        camera.native_format()
    );

    let mut frame = vec![0; PixelFormat::Rgb.frame_size(width, height)];
    for frame_idx in 0..100 {
        let color = (frame_idx % 255) as u8;
        for pixel in frame.chunks_exact_mut(3) {
            pixel.copy_from_slice(&[color, 255 - color, color / 2]);
        }

        camera.send(&frame)?;
        std::thread::sleep(Duration::from_secs_f64(1.0 / fps));
    }

    camera.close();
    Ok(())
}
