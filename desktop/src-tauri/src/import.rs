use std::path::Path;

use tiny_skia::Pixmap;

use crate::gallery::GalleryError;

/// Decode an image file into a premultiplied-alpha tiny-skia `Pixmap`.
pub(crate) fn load_image(path: &Path) -> Result<Pixmap, GalleryError> {
    let img = image::open(path)?;
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let mut data = rgba.into_raw();
    premultiply_rgba(&mut data);

    let size = tiny_skia::IntSize::from_wh(w, h).ok_or(GalleryError::InvalidImage)?;
    Pixmap::from_vec(data, size).ok_or(GalleryError::InvalidImage)
}

/// Convert straight-alpha RGBA to premultiplied-alpha in place.
fn premultiply_rgba(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(4) {
        let a = pixel[3] as u16;
        if a == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        } else if a < 255 {
            pixel[0] = ((pixel[0] as u16 * a + 127) / 255) as u8;
            pixel[1] = ((pixel[1] as u16 * a + 127) / 255) as u8;
            pixel[2] = ((pixel[2] as u16 * a + 127) / 255) as u8;
        }
    }
}
