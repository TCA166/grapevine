mod app;
use std::{net::SocketAddr, str::FromStr};

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
                SocketAddr::from_str("0.0.0.0:5000").unwrap(),
            ))))
        }),
    )
}
