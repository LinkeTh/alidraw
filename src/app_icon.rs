use eframe::egui;

/// The same 7 rainbow colours used in the toolbar header stripe.
const RAINBOW_STRIPE: [(u8, u8, u8); 7] = [
    (255, 59, 48),
    (255, 149, 0),
    (255, 204, 0),
    (76, 217, 100),
    (90, 200, 250),
    (0, 122, 255),
    (175, 82, 222),
];

const ICON_SIZE: u32 = 64;

/// Generates a 64x64 rainbow-stripe icon suitable for the application window.
pub(crate) fn generate_rainbow_icon() -> egui::IconData {
    let pixel_count = (ICON_SIZE * ICON_SIZE) as usize;
    let mut rgba = Vec::with_capacity(pixel_count * 4);

    let band_count = RAINBOW_STRIPE.len() as u32;
    let band_height = ICON_SIZE / band_count; // 9 px per band

    for y in 0..ICON_SIZE {
        let band_index = (y / band_height).min(band_count - 1) as usize;
        let (r, g, b) = RAINBOW_STRIPE[band_index];

        // Round the corners: skip pixels that fall outside a circle-ish mask
        // with a corner radius of 12 pixels.
        for x in 0..ICON_SIZE {
            let alpha = corner_alpha(x, y, ICON_SIZE, 12);
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(alpha);
        }
    }

    egui::IconData {
        rgba,
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

/// Returns 255 if the pixel is inside the rounded rect, 0 if outside.
/// Uses a simple distance check at each corner.
fn corner_alpha(x: u32, y: u32, size: u32, radius: u32) -> u8 {
    let last = size - 1;
    let r = radius as f32;

    // Only check corners
    let (cx, cy) = if x < radius && y < radius {
        (radius as f32, radius as f32) // top-left
    } else if x > last - radius && y < radius {
        ((last - radius) as f32, radius as f32) // top-right
    } else if x < radius && y > last - radius {
        (radius as f32, (last - radius) as f32) // bottom-left
    } else if x > last - radius && y > last - radius {
        ((last - radius) as f32, (last - radius) as f32) // bottom-right
    } else {
        return 255; // not in a corner zone
    };

    let dx = x as f32 - cx;
    let dy = y as f32 - cy;
    if dx * dx + dy * dy <= r * r { 255 } else { 0 }
}
