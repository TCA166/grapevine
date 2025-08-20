use std::{sync::Arc, thread};

use super::{channel::Channel, listener::listener_thread, message::Message};

pub struct GrapevineApp {
    listener_thread: thread::JoinHandle<()>,
    channels: Arc<Vec<Channel>>,
}

impl GrapevineApp {
    pub fn new(address: String) -> Self {
        Self {
            listener_thread: thread::spawn(move || listener_thread(address)),
            channels: Arc::new(Vec::new()),
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
