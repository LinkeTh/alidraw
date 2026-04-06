#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to save image to {path}: {source}")]
    ExportImage {
        path: String,
        #[source]
        source: image::ImageError,
    },
}

impl AppError {
    pub fn export_image(path: &std::path::Path, source: image::ImageError) -> Self {
        Self::ExportImage {
            path: path.display().to_string(),
            source,
        }
    }
}
