use std::any::type_name;

mod app;

mod protocol;

mod handler;

mod ui;
use ui::GrapevineUI;

mod settings;
use settings::Settings;

mod modals;

mod widgets;

use serde_json::from_str;

const TITLE: &'static str = env!("CARGO_PKG_NAME");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 1000.0])
            .with_title(TITLE),
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|cc| {
            Ok({
                let settings = cc
                    .storage
                    .and_then(|storage| {
                        storage
                            .get_string(type_name::<Settings>())
                            .and_then(|serialized| from_str(serialized.as_str()).unwrap())
                    })
                    .unwrap_or(Settings::default());
                Box::new(GrapevineUI::new(settings))
            })
        }),
    )
}
