# Alidraw contributor guide

Alidraw is a child-friendly Tauri 2 drawing app for Linux and Windows. See the [project README](README.md) for setup, development commands, platform dependencies, and release artifacts.

## Drawing and layout

- Start in a decorated 1366×900 window. Keep the paper white, the rainbow toolbar on the right, and the action buttons pinned at the bottom.
- Make tools easy for a four-year-old to use with a mouse or tablet pen. Keep controls large, visible, and free of nested menus.
- Support brush, eraser, fill, three brush styles, 24 colors, 20 brush sizes, and ten undo steps. Save with one tap to the Pictures/Alidraw gallery and open pictures by thumbnail. Keep confirmation for destructive actions.
- Keep file ownership in Rust. Update an existing Alidraw PNG on Save. Treat adult-added PNGs and JPEGs as originals and save edits to a separate drawing.
- Keep marks anchored at the top left as the paper grows. Resizing to a smaller window hides the edges. Export the whole sheet.
- Pressure sensitivity, autosave, shapes, stamps, printing, and audio are out of scope.

## Implementation and verification

- Work in `desktop/src/` for the React controls and live canvas preview. Keep pointer movement independent of React renders and Rust calls.
- Work in `desktop/src-tauri/src/` for native commands, the committed bitmap, history, and file handling. Apply edits in input order.
- Follow the [Rust rules](RUST_RULES.md). Use edition 2024, borrow where practical, propagate errors with `?`, use `thiserror` for crate errors, and avoid `.unwrap()` in production code.
- Run the frontend build, Rust formatting check, tests, and Clippy with warnings denied before committing. Use `./build-all.sh` from the root to verify Linux and Windows releases.
