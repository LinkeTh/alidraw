use std::path::{Path, PathBuf};

use rfd::FileDialog;
use tiny_skia::Pixmap;

use crate::error::AppError;

/// Show a native file-open dialog for PNG / JPEG images.
pub(crate) fn choose_import_path() -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Open image")
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .pick_file()
}

/// Decode an image file into a premultiplied-alpha tiny-skia `Pixmap`.
pub(crate) fn load_image(path: &Path) -> Result<Pixmap, AppError> {
    let img = image::open(path).map_err(|e| AppError::import_image(path, e))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let mut data = rgba.into_raw();
    premultiply_rgba(&mut data);

    let mut pixmap = Pixmap::new(w, h).ok_or_else(|| {
        AppError::import_image(
            path,
            image::ImageError::Limits(image::error::LimitError::from_kind(
                image::error::LimitErrorKind::DimensionError,
            )),
        )
    })?;
    pixmap.data_mut().copy_from_slice(&data);
    Ok(pixmap)
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
