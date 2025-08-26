mod app;
use std::net::Ipv4Addr;

use app::GrapevineApp;

mod protocol;

mod handler;

mod ui;

use ui::GrapevineUI;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|cc| {
            Ok(Box::new(GrapevineUI::new(GrapevineApp::new(
                Ipv4Addr::new(0, 0, 0, 0),
                5000,
            ))))
        }),
    )
}
