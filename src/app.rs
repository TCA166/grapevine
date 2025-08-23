use std::{
    sync::{Arc, Mutex},
    thread,
};

use super::{channel::Channel, listener::listener_thread};

pub struct GrapevineApp {
    listener_thread: thread::JoinHandle<()>,
    channels: Arc<Mutex<Vec<Arc<Channel>>>>,
}

impl GrapevineApp {
    pub fn new(address: String) -> Self {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let channels_clone = channels.clone();
        Self {
            listener_thread: thread::spawn(move || listener_thread(address, channels_clone)),
            channels,
        }
    }
}

impl Default for GrapevineApp {
    fn default() -> Self {
        Self::new("0.0.0.0:5000".to_owned())
    }
}

impl eframe::App for GrapevineApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("Channels").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                for channel in self.channels.lock().unwrap().iter() {
                    ui.label(channel.name());
                }
                ui.menu_button("New Channel", |ui| {});
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {});
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
