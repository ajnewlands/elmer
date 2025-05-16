//#![windows_subsystem = "windows"]
mod gui;
mod rabbit;
use eframe::egui::{self, viewport, Color32, IconData};
use rabbit::ConnectionManager;

static ICON: &[u8] = include_bytes!("../icons/elmer.ico");

fn load_icon_data() -> IconData {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(ICON)).expect("infallible");
    let icon = icon_dir.entries().first().expect("infallible");
    let mut rgba: Vec<u8> = icon.data().to_vec();
    rgba.chunks_exact_mut(4).for_each(|chunk| chunk.swap(0, 2));
    let icon_data = IconData {
        width: icon.width(),
        height: icon.height(),
        rgba,
    };

    icon_data
}

fn main() {
    env_logger::init();

    let icon_data = load_icon_data();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    let connection_manager = ConnectionManager::new();

    if let Err(e) = eframe::run_native(
        "Elmer",
        native_options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(gui::App::new(cc, connection_manager)))
        }),
    ) {
        log::error!("Failed to instantiate GUI: {}", e);
        std::process::exit(1);
    }
}
