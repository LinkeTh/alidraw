#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gallery;
mod import;
mod ink;
mod paper;

use gallery::Catalog;
use paper::Paper;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};
use tauri::{AppHandle, Manager, State, ipc::Response};
use uuid::Uuid;

#[derive(Default)]
struct Document {
    paper: Paper,
    saved_id: Option<Uuid>,
}

impl Document {
    fn new_sheet(&mut self, width: u32, height: u32) -> Result<Response, String> {
        let patch = self.paper.replace(width, height, None)?;
        self.saved_id = None;
        Ok(Response::new(patch))
    }

    fn open(&mut self, path: &Path, width: u32, height: u32) -> Result<Response, String> {
        let image = import::load_image(path).map_err(|e| e.to_string())?;
        let patch = self.paper.replace(width, height, Some(&image))?;
        self.saved_id = gallery::managed_id(path);
        Ok(Response::new(patch))
    }

    fn save(&mut self, dir: &Path) -> Result<(), String> {
        self.saved_id = Some(
            gallery::save(dir, self.saved_id, self.paper.image()?).map_err(|e| e.to_string())?,
        );
        Ok(())
    }
}

#[derive(Default)]
struct Drawing {
    document: Mutex<Document>,
    catalog: Mutex<Catalog>,
}

impl Drawing {
    fn document(&self) -> Result<MutexGuard<'_, Document>, String> {
        self.document.lock().map_err(|e| e.to_string())
    }

    fn entry(&self, id: Uuid) -> Result<PathBuf, String> {
        self.catalog
            .lock()
            .map_err(|e| e.to_string())?
            .path(id)
            .map_err(|e| e.to_string())
    }
}

fn gallery_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let paths = app.path();
    paths
        .picture_dir()
        .or_else(|_| paths.home_dir().map(|d| d.join("Pictures")))
        .map(|d| d.join("Alidraw"))
        .map_err(|e| e.to_string())
}

async fn on_worker<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn grow(width: u32, height: u32, state: State<'_, Drawing>) -> Result<(), String> {
    state.document()?.paper.grow(width, height)
}
#[tauri::command]
async fn commit(
    points: Vec<[f32; 2]>,
    width: f32,
    rgba: [u8; 4],
    state: State<'_, Drawing>,
) -> Result<Response, String> {
    state
        .document()?
        .paper
        .commit(points, width, rgba)
        .map(Response::new)
}
#[tauri::command]
async fn fill(
    point: [f32; 2],
    color: [u8; 3],
    state: State<'_, Drawing>,
) -> Result<Response, String> {
    state
        .document()?
        .paper
        .fill(point, color)
        .map(Response::new)
}
#[tauri::command]
async fn history(redo: bool, state: State<'_, Drawing>) -> Result<Response, String> {
    state.document()?.paper.history(redo).map(Response::new)
}
#[tauri::command]
async fn new_paper(width: u32, height: u32, state: State<'_, Drawing>) -> Result<Response, String> {
    state.document()?.new_sheet(width, height)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GalleryView {
    entries: Vec<Uuid>,
    confirm_open: bool,
}

#[tauri::command]
async fn list_drawings(app: AppHandle) -> Result<GalleryView, String> {
    on_worker(move || {
        let state = app.state::<Drawing>();
        let (catalog, entries) = Catalog::scan(&gallery_dir(&app)?).map_err(|e| e.to_string())?;
        *state.catalog.lock().map_err(|e| e.to_string())? = catalog;
        let confirm_open = state.document()?.paper.needs_confirmation();
        Ok(GalleryView {
            entries,
            confirm_open,
        })
    })
    .await
}

#[tauri::command]
async fn drawing_thumbnail(id: Uuid, app: AppHandle) -> Result<Response, String> {
    on_worker(move || {
        let path = app.state::<Drawing>().entry(id)?;
        gallery::thumbnail(&path)
            .map(Response::new)
            .map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn open_image(id: Uuid, width: u32, height: u32, app: AppHandle) -> Result<Response, String> {
    on_worker(move || {
        let state = app.state::<Drawing>();
        let path = state.entry(id)?;
        state.document()?.open(&path, width, height)
    })
    .await
}

#[tauri::command]
async fn save_image(app: AppHandle) -> Result<(), String> {
    on_worker(move || {
        let directory = gallery_dir(&app)?;
        app.state::<Drawing>().document()?.save(&directory)
    })
    .await
}

#[cfg(target_os = "linux")]
fn configure_linux_graphics() -> std::io::Result<()> {
    use std::{env, os::unix::process::CommandExt, path::Path, process::Command};

    const SHARED_BUFFERS: &str = "WEBKIT_DMABUF_RENDERER_FORCE_SHM";
    if Path::new("/sys/module/nvidia_drm").is_dir() && env::var_os(SHARED_BUFFERS).is_none() {
        // WebKitGTK can select a GBM format the NVIDIA driver cannot allocate.
        // Re-exec with shared-memory buffer transport before GTK or WebKit starts.
        let mut args = env::args_os();
        return Err(Command::new("/proc/self/exe")
            .arg0(args.next().unwrap_or_else(|| "alidraw-desktop".into()))
            .args(args)
            .env(SHARED_BUFFERS, "1")
            .exec());
    }
    gdk::set_allowed_backends("x11,wayland");
    Ok(())
}

fn main() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    configure_linux_graphics()?;
    tauri::Builder::default()
        .manage(Drawing::default())
        .invoke_handler(tauri::generate_handler![
            grow,
            commit,
            fill,
            history,
            new_paper,
            list_drawings,
            drawing_thumbnail,
            open_image,
            save_image
        ])
        .run(tauri::generate_context!())
        .expect("could not start Alidraw");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use std::{
        fs,
        time::{Duration, SystemTime},
    };

    #[test]
    fn gallery_save_update_restart_and_new_drawing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("Alidraw");
        assert!(Catalog::scan(&dir).expect("created folder").1.is_empty());

        let mut first = Document::default();
        first.new_sheet(64, 48).expect("paper");
        first.save(&dir).expect("save");
        first
            .paper
            .commit(vec![[60.0, 44.0]], 4.0, [1, 2, 3, 255])
            .expect("last stroke");
        first.save(&dir).expect("update");
        let (catalog, ids) = Catalog::scan(&dir).expect("scan");
        assert_eq!(ids.len(), 1);
        let original = catalog.path(ids[0]).expect("entry");
        let mut restarted = Document::default();
        restarted
            .open(&original, 10, 10)
            .expect("open in small window");
        let img = restarted.paper.image().expect("paper");
        assert_eq!((img.width(), img.height()), (64, 48));
        assert_eq!(img.pixel(60, 44).expect("hidden stroke").red(), 1);
        restarted
            .paper
            .fill([0.0, 0.0], [20, 30, 40])
            .expect("edit");
        restarted.save(&dir).expect("update after restart");
        assert_eq!(
            image::open(&original)
                .expect("saved image")
                .to_rgb8()
                .get_pixel(0, 0)
                .0,
            [20, 30, 40]
        );
        restarted.new_sheet(10, 10).expect("new");
        restarted.save(&dir).expect("second drawing");
        assert_eq!(Catalog::scan(&dir).expect("scan").1.len(), 2);

        fs::File::options()
            .write(true)
            .open(&original)
            .expect("file")
            .set_modified(SystemTime::now() - Duration::from_secs(60))
            .expect("older original");
        assert_ne!(catalog_path_first(&dir), original);
        restarted.open(&original, 10, 10).expect("reopen");
        // Use an older timestamp on the other picture, including on coarse-resolution filesystems.
        fs::File::options()
            .write(true)
            .open(catalog_path_first(&dir))
            .expect("second file")
            .set_modified(SystemTime::now() - Duration::from_secs(30))
            .expect("age second");
        restarted.save(&dir).expect("resave");
        assert_eq!(catalog_path_first(&dir), original);
    }

    fn catalog_path_first(dir: &Path) -> PathBuf {
        let (catalog, ids) = Catalog::scan(dir).expect("scan");
        catalog.path(ids[0]).expect("first entry")
    }

    #[test]
    fn adult_images_get_independent_png_copies() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();
        let mut doc = Document::default();
        for name in ["Coloring page.JPEG", "Fox.png"] {
            let source = dir.join(name);
            RgbImage::from_pixel(500, 250, Rgb([25u8, 100, 170]))
                .save(&source)
                .expect("original");
            let bytes = fs::read(&source).expect("bytes");
            assert!(gallery::managed_id(&source).is_none());
            let thumb = gallery::thumbnail(&source).expect("preview");
            let thumb = image::load_from_memory(&thumb).expect("read preview");
            assert_eq!((thumb.width(), thumb.height()), (320, 160));
            doc.open(&source, 320, 240).expect("open");
            doc.paper
                .commit(vec![[10.0, 10.0]], 8.0, [0, 0, 0, 255])
                .expect("edit");
            doc.save(dir).expect("save as new");
            doc.save(dir).expect("update copy");
            assert_eq!(fs::read(&source).expect("original bytes"), bytes);
        }
        assert_eq!(Catalog::scan(dir).expect("scan").1.len(), 4);
    }

    #[test]
    fn unavailable_files_leave_document_recoverable() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();
        let broken = dir.join("bad.png");
        fs::write(&broken, b"invalid").expect("broken fixture");
        fs::write(dir.join("notes.txt"), b"text").expect("other file");
        fs::create_dir(dir.join("folder.jpg")).expect("subfolder");
        let (catalog, ids) = Catalog::scan(dir).expect("scan");
        assert_eq!(ids.len(), 1);
        assert!(catalog.path(Uuid::new_v4()).is_err());
        assert!(gallery::thumbnail(&broken).is_err());

        let mut doc = Document::default();
        doc.new_sheet(7, 9).expect("paper");
        doc.paper.fill([0.0, 0.0], [0, 0, 0]).expect("fill");
        assert!(doc.save(&broken).is_err());
        assert!(doc.saved_id.is_none());
        doc.save(dir).expect("retry save");
        let saved = doc.saved_id;
        assert!(doc.open(&broken, 1, 1).is_err());
        assert_eq!(doc.saved_id, saved);
        doc.paper.history(false).expect("undo still works");
        assert!(
            doc.paper
                .image()
                .expect("paper")
                .data()
                .iter()
                .all(|&b| b == 255)
        );
        fs::remove_file(&broken).expect("remove picture");
        assert!(catalog.path(ids[0]).is_err());
    }
}
