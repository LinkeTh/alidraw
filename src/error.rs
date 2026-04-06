use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("failed to save image to {}: {source}", path.display())]
    ExportImage {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
}

impl AppError {
    pub(crate) fn export_image(path: &std::path::Path, source: image::ImageError) -> Self {
        Self::ExportImage {
            path: path.to_path_buf(),
            source,
        }
    }
}
