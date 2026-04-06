use std::path::{Path, PathBuf};

use image::ImageFormat;
use rfd::FileDialog;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportFormat {
    Png,
    Jpeg,
}

impl ExportFormat {
    fn from_extension(extension: Option<&str>) -> Self {
        match extension.map(|ext| ext.to_ascii_lowercase()) {
            Some(ext) if matches!(ext.as_str(), "jpg" | "jpeg") => Self::Jpeg,
            _ => Self::Png,
        }
    }

    fn image_format(self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Jpeg => ImageFormat::Jpeg,
        }
    }

    fn default_extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

pub(crate) fn choose_export_path() -> Option<(PathBuf, ExportFormat)> {
    // We try PNG first, then JPEG.  rfd does not expose which filter
    // the user selected, so we infer format from the resulting path
    // extension and fall back to PNG when no extension is present.
    let path = FileDialog::new()
        .set_title("Save drawing")
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"])
        .set_file_name("drawing.png")
        .save_file()?;

    let format = ExportFormat::from_extension(path.extension().and_then(|ext| ext.to_str()));
    Some((path, format))
}

pub(crate) fn save_rgba_image(
    width: u32,
    height: u32,
    rgba: &[u8],
    path: &Path,
    format: ExportFormat,
) -> Result<PathBuf, AppError> {
    let mut target_path = path.to_path_buf();

    if target_path.extension().is_none() {
        target_path.set_extension(format.default_extension());
    }

    let (buffer, color_type) = match format {
        ExportFormat::Png => (rgba.to_vec(), image::ColorType::Rgba8),
        ExportFormat::Jpeg => (rgba_to_rgb_over_white(rgba), image::ColorType::Rgb8),
    };

    image::save_buffer_with_format(
        &target_path,
        &buffer,
        width,
        height,
        color_type,
        format.image_format(),
    )
    .map_err(|source| AppError::export_image(&target_path, source))?;

    Ok(target_path)
}

fn rgba_to_rgb_over_white(rgba: &[u8]) -> Vec<u8> {
    let pixel_count = rgba.len() / 4;
    let mut out = Vec::with_capacity(pixel_count * 3);
    for pixel in rgba.chunks_exact(4) {
        let alpha = pixel[3] as u16;
        let inverse_alpha = 255_u16.saturating_sub(alpha);
        let blend =
            |component: u8| ((component as u16 * alpha + 255 * inverse_alpha + 127) / 255) as u8;
        out.extend_from_slice(&[blend(pixel[0]), blend(pixel[1]), blend(pixel[2])]);
    }
    out
}
