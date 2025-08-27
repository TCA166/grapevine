use egui::{Frame, Response, TextBuffer, Ui, Widget};
use egui_file::FileDialog;

pub struct FilePathInput<'input, 'dialog, S: TextBuffer> {
    input: &'input mut S,
    name: &'static str,
    dialog: &'dialog mut Option<FileDialog>,
}

impl<'input, 'dialog, S: TextBuffer> FilePathInput<'input, 'dialog, S> {
    pub fn new(
        input: &'input mut S,
        name: &'static str,
        dialog: &'dialog mut Option<FileDialog>,
    ) -> Self {
        Self {
            input,
            name,
            dialog,
        }
    }
}

impl<S: TextBuffer> Widget for FilePathInput<'_, '_, S> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::new()
            .show(ui, |ui| {
                ui.label(self.name);

                let resp = ui.horizontal(|ui| {
                    ui.text_edit_singleline(self.input);

                    if ui.button("📁").clicked() {
                        self.dialog.open();
                    }
                });
            })
            .response
    }
}
