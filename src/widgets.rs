use std::{fs::read_dir, path::Path};

use egui::{
    Frame, MenuBar, PopupCloseBehavior, Response, TextBuffer, Ui, Widget,
    containers::menu::MenuConfig,
};

pub struct FilePathInput<'input, 'path, S: TextBuffer> {
    input: &'input mut S,
    name: &'static str,
    default_path: &'path Path,
}

impl<'input, 'path, S: TextBuffer> FilePathInput<'input, 'path, S> {
    pub fn new<P: AsRef<Path>>(
        input: &'input mut S,
        name: &'static str,
        default_path: &'path P,
    ) -> Self {
        Self {
            input,
            name,
            default_path: default_path.as_ref(),
        }
    }

    fn picker_widget(self, ui: &mut Ui) {
        let mut search_path = Path::new(self.input.as_str());
        if !search_path.exists() {
            search_path = self.default_path;
        } else if !search_path.is_dir() {
            search_path = search_path.parent().unwrap_or(self.default_path);
        }

        if let Some(parent) = search_path.parent() {
            if ui.button("⤴").clicked() {
                self.input.replace_with(parent.to_string_lossy().as_ref());
            }
        }

        if let Ok(iter) = read_dir(search_path) {
            for ent in iter {
                if let Ok(ent) = ent {
                    let path = ent.path();

                    let icon = if path.is_file() { "" } else { "📁" };

                    if ui
                        .button(format!("{} {}", icon, ent.file_name().display()))
                        .clicked()
                    {
                        self.input.replace_with(path.to_string_lossy().as_ref());
                        if path.is_file() {
                            ui.close();
                        }
                    }
                }
            }
        }
    }
}

impl<S: TextBuffer> Widget for FilePathInput<'_, '_, S> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::new()
            .show(ui, |ui| {
                ui.label(self.name);
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(self.input);

                    MenuBar::new()
                        .config(
                            MenuConfig::default()
                                .close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                        )
                        .ui(ui, |ui| {
                            ui.menu_button("📂", |ui| {
                                self.picker_widget(ui);
                            })
                        })
                });
            })
            .response
    }
}
