use eframe::egui;

mod app;
use app::HihoApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "hiho - Менеджер паролей",
        native_options,
        Box::new(|_cc| {
            Box::<HihoApp>::default()
        }),
    )
}