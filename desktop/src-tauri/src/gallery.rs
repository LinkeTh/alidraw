use std::{
    collections::HashMap,
    fs,
    io::{self, BufWriter, Cursor, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use image::{ImageEncoder, ImageFormat, codecs::png::PngEncoder};
use tempfile::NamedTempFile;
use tiny_skia::Pixmap;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GalleryError {
    #[error("Gallery file error: {0}")]
    File(#[from] io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid drawing dimensions")]
    InvalidImage,
    #[error("This picture is no longer available")]
    Missing,
}

fn image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| {
            ["png", "jpg", "jpeg"]
                .iter()
                .any(|format| ext.eq_ignore_ascii_case(format))
        })
}

// Each scan issues fresh handles. The frontend cannot supply a filesystem path.
#[derive(Default)]
pub(crate) struct Catalog(HashMap<Uuid, PathBuf>);

impl Catalog {
    pub(crate) fn scan(dir: &Path) -> Result<(Self, Vec<Uuid>), GalleryError> {
        fs::create_dir_all(dir)?;
        let mut files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !image_file(&path) || !entry.file_type().is_ok_and(|ty| ty.is_file()) {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            files.push((modified, path));
        }
        files.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let mut catalog = Self::default();
        let mut ids = Vec::with_capacity(files.len());
        for (_, path) in files {
            let id = Uuid::new_v4();
            catalog.0.insert(id, path);
            ids.push(id);
        }
        Ok((catalog, ids))
    }

    pub(crate) fn path(&self, id: Uuid) -> Result<PathBuf, GalleryError> {
        let path = self.0.get(&id).ok_or(GalleryError::Missing)?;
        if !fs::symlink_metadata(path)?.is_file() {
            return Err(GalleryError::Missing);
        }
        Ok(path.clone())
    }
}

pub(crate) fn managed_id(path: &Path) -> Option<Uuid> {
    let name = path.file_name()?.to_str()?;
    let id = Uuid::parse_str(name.strip_prefix("alidraw-")?.strip_suffix(".png")?).ok()?;
    (name == format!("alidraw-{id}.png")).then_some(id)
}

pub(crate) fn thumbnail(path: &Path) -> Result<Vec<u8>, GalleryError> {
    let preview = image::open(path)?.thumbnail(320, 240);
    let mut bytes = Cursor::new(Vec::new());
    preview.write_to(&mut bytes, ImageFormat::Png)?;
    Ok(bytes.into_inner())
}

pub(crate) fn save(
    dir: &Path,
    previous: Option<Uuid>,
    paper: &Pixmap,
) -> Result<Uuid, GalleryError> {
    fs::create_dir_all(dir)?;
    let id = previous.unwrap_or_else(Uuid::new_v4);
    let destination = dir.join(format!("alidraw-{id}.png"));
    let mut temp = NamedTempFile::new_in(dir)?;
    {
        let mut writer = BufWriter::new(temp.as_file_mut());
        PngEncoder::new(&mut writer).write_image(
            paper.data(),
            paper.width(),
            paper.height(),
            image::ExtendedColorType::Rgba8,
        )?;
        writer.flush()?;
    }
    temp.as_file().sync_all()?;
    if previous.is_some() {
        temp.persist(destination)
    } else {
        temp.persist_noclobber(destination)
    }
    .map_err(|e| e.error)?;
    Ok(id)
}
