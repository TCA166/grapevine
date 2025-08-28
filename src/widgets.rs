use egui::{Frame, Response, Style, TextBuffer, Ui, Widget};

pub struct FilePathInput<'input, 'dnd, S: TextBuffer> {
    input: &'input mut S,
    name: &'static str,
    dnd: &'dnd mut bool,
}

impl<'input, 'dnd, S: TextBuffer> FilePathInput<'input, 'dnd, S> {
    pub fn new(input: &'input mut S, name: &'static str, dnd: &'dnd mut bool) -> Self {
        Self { input, name, dnd }
    }
}

impl<S: TextBuffer> Widget for FilePathInput<'_, '_, S> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.ctx().input(|input| {
            if input.pointer.any_click() {
                *self.dnd = false;
            }
        });

        ui.label(self.name);
        if *self.dnd {
            Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.label("Drag and drop here");

                ui.input(|input| eprintln!("{:?}", input.raw.dropped_files));
            })
        } else {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(self.input);

                if ui.button("📁").clicked() {
                    *self.dnd = true;
                }
            })
        }
        .response
    }
}
