# Alidraw

Alidraw is a drawing app for young children on Linux and Windows. A white sheet fills the left side of the window. The rainbow toolbar on the right keeps brushes, colors, and file actions within reach of a mouse or tablet pen.

## Drawing

- Choose **Brush** and draw with a pencil, marker, or watercolor brush. Pick from 24 colors and use the vertical slider to choose one of 20 brush sizes. **Eraser** paints over marks with white. **Fill** colors a connected area.
- Enlarge the window to add paper along the right and bottom edges. Shrinking the window hides those edges. **Save** includes the entire sheet. Press **F11** to toggle fullscreen.
- **Undo** and **Redo** step through the last ten edits.
- Tap **Save** to keep the drawing. The first save creates a PNG with an automatic name. Later saves update that picture, even after reopening it. **New** starts a separate drawing.
- Tap **Open** to see **My drawings**, with the most recently changed pictures first. Tap a picture to continue it, or choose **Back to drawing**. Alidraw asks before replacing a drawing that contains work.

## Where drawings live

Alidraw creates an `Alidraw` folder in the current user's Pictures directory. If the OS has no configured Pictures location, it uses `Pictures/Alidraw` under the home directory. Back up this folder to keep the gallery.

Adults can add PNGs, JPGs, and JPEGs directly to the folder, including coloring pages. Saving an edit to an added image creates a separate PNG and preserves the original. Alidraw recognizes its own drawings by the filename `alidraw-<uuid>.png`. Images open at their original size, anchored at the top left. A smaller window can hide part of a picture, and Save still includes the whole sheet.

## Run from source

Install Node.js 22.12 or later, npm, a stable Rust toolchain with edition 2024 support, and the native dependencies for your platform. Linux development uses GTK 3 and WebKitGTK 4.1. Windows development uses the MSVC C++ Build Tools and the WebView2 runtime. [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/)

Start the app from the repository root with these commands.

```sh
cd desktop
npm ci
npm run tauri dev
```

## Build a release

To build the Linux executable, Windows x64 executable, and Windows NSIS installer on a Linux machine, install `cargo-xwin`, the `x86_64-pc-windows-msvc` Rust target, Clang/LLVM with LLD, and NSIS. The first Windows build downloads the SDK and CRT. [Tauri Windows cross-build guide](https://v2.tauri.app/distribute/windows-installer/#build-windows-apps-on-linux-and-macos)

```sh
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
./build-all.sh
```

The script prints the artifact paths. The default target directory is `desktop/src-tauri/target/`.

| Artifact | Location within the target directory |
| --- | --- |
| Linux executable | `release/alidraw-desktop` |
| Windows executable | `x86_64-pc-windows-msvc/release/alidraw-desktop.exe` |
| Windows installer | `x86_64-pc-windows-msvc/release/bundle/nsis/` |

## Development

The React and TypeScript frontend lives in `desktop/src/`. It draws the active stroke as pointer events arrive. The Tauri 2 backend in `desktop/src-tauri/src/` commits each completed stroke to the tiny-skia bitmap, manages undo history, and opens and saves image files.

Run the frontend and Rust checks from the repository root.

```sh
cd desktop
npm ci
npm run build
cd src-tauri
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

For pen latency diagnostics, **F8** displays input and draw-call timing after each stroke. **F9** opens a temporary drawing layer that bypasses Rust commits. Press it again to discard the test marks and return to the drawing.

On Linux, Alidraw prefers X11 when available. With the NVIDIA DRM driver loaded, it uses WebKitGTK’s shared-memory buffer transport to avoid a black window when GBM buffer allocation fails. To test the hardware-buffer path, launch with `WEBKIT_DMABUF_RENDERER_FORCE_SHM=0`.
