use std::any::type_name;

mod handler;

mod ui;
use ui::GrapevineUI;

mod settings;
use settings::Settings;

mod modals;

use grapevine_lib::ChannelDesc;
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
                let mut settings = None;
                let mut saved_channels = None;

                if let Some(storage) = cc.storage {
                    if let Some(serialized_settings) = storage.get_string(type_name::<Settings>()) {
                        match from_str(serialized_settings.as_str()) {
                            Ok(val) => {
                                settings = Some(val);
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }

                    if let Some(serialized_channels) =
                        storage.get_string(type_name::<ChannelDesc>())
                    {
                        match from_str(serialized_channels.as_str()) {
                            Ok(val) => saved_channels = Some(val),
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                }

                Box::new(GrapevineUI::new(
                    settings.unwrap_or(Settings::default()),
                    saved_channels.unwrap_or(Vec::default()),
                ))
            })
        }),
    )
}
