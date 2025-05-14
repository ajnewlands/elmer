mod gui;

use eframe::egui::{self, Color32};
fn main() {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Elmer",
        native_options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(gui::App::new(cc)))
        }),
    ) {
        log::error!("Failed to instantiate GUI: {}", e);
        std::process::exit(1);
    }
}
