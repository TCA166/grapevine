use std::error;

use egui::{Context, Id, Modal, Ui};
use egui_notify::Toasts;

pub trait Form<'a>: Default {
    type Args;
    type Error: error::Error;
    type Ret;

    fn show(&mut self, ui: &mut Ui, args: Self::Args) -> Result<Option<Self::Ret>, Self::Error>;
}

pub struct ModalForm<T> {
    inner: T,
    modal_name: Id,
    toasts: Toasts,
}

impl<'a, T: Form<'a>> ModalForm<T> {
    pub fn new(inner: T, modal_name: &'static str) -> Self {
        Self {
            inner,
            modal_name: modal_name.into(),
            toasts: Toasts::default(),
        }
    }

    pub fn show(&mut self, ctx: &Context, args: T::Args) -> Option<T::Ret> {
        self.toasts.show(ctx);

        Modal::new(self.modal_name)
            .show(ctx, |ui| self.inner.show(ui, args))
            .inner
            .unwrap_or_else(|err| {
                self.toasts.error(format!("{}", &err));
                None
            })
    }
}
