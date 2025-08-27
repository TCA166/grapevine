mod app;

mod protocol;

mod handler;

mod ui;

mod settings;

mod modals;

mod widgets;

use ui::GrapevineUI;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|cc| Ok(Box::new(GrapevineUI::new()))),
    )
}
