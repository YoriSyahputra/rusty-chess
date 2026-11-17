use eframe::egui;
use rusty_chess::app::RustyChessApp;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let icon_data = build_procedural_app_icon();

    let viewport = egui::ViewportBuilder::default()
        .with_title("Rusty Chess — Native Grandmaster AI Persona Chess")
        .with_app_id("rusty-chess")
        .with_inner_size([1180.0, 780.0])
        .with_min_inner_size([960.0, 640.0])
        .with_icon(Arc::new(icon_data));

    let native_options = eframe::NativeOptions {
        viewport,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Chess",
        native_options,
        Box::new(|cc| Ok(Box::new(RustyChessApp::new(cc)))),
    )
}

/// Menghasilkan ikon RGBA 32x32 secara prosedural di memori sehingga aplikasi 100% mandiri.
fn build_procedural_app_icon() -> egui::IconData {
    let width = 32usize;
    let height = 32usize;
    let mut rgba = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= 15.0 {
                // Dark emerald chess medallion background
                let is_border = dist >= 13.2;
                let (mut r, mut g, mut b) = if is_border {
                    (220u8, 185u8, 75u8)
                } else if ((x / 8) + (y / 8)) % 2 == 0 {
                    (42u8, 56u8, 44u8)
                } else {
                    (30u8, 40u8, 32u8)
                };

                // Central white knight/crown silhouette
                let in_pedestal = (22..=25).contains(&y) && (9..=22).contains(&x);
                let in_body = (12..22).contains(&y) && (12..=20).contains(&x);
                let in_head = (8..=15).contains(&y) && (9..=18).contains(&x);
                if in_pedestal || in_body || in_head {
                    r = 245;
                    g = 244;
                    b = 238;
                }

                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }
    }

    egui::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    }
}
