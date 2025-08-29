use std::ops::{Deref, DerefMut};

use egui::Context;
use egui_notify::Toasts;

use grapevine_lib::EventRecipient;

#[derive(Default)]
pub struct UiEventHandler {
    toasts: Toasts,
}

impl UiEventHandler {
    pub fn ui(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }
}

impl Deref for UiEventHandler {
    type Target = Toasts;

    fn deref(&self) -> &Self::Target {
        &self.toasts
    }
}

impl DerefMut for UiEventHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.toasts
    }
}

impl EventRecipient for UiEventHandler {
    fn info(&mut self, message: &str) {
        self.toasts.info(message);
    }

    fn warn(&mut self, message: &str) {
        self.toasts.warning(message);
    }

    fn error(&mut self, message: &str) {
        self.toasts.error(message);
    }

    fn success(&mut self, message: &str) {
        self.toasts.success(message);
    }
}
