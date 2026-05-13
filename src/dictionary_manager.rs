use crate::config::{DictionaryEntry, DictionaryLanguage, DictionaryPriority};
use crate::dictionary_store::DictionaryStore;
use eframe::egui;
use std::sync::Arc;

const MANAGER_VIEWPORT_ID: &str = "nputella_dictionary_manager";

#[derive(Clone)]
struct EditableEntry {
    written: String,
    aliases: String,
    phonetic: bool,
    high_priority: bool,
    language: DictionaryLanguage,
}

pub struct DictionaryManager {
    entries: Vec<EditableEntry>,
    selected: Option<usize>,
    loaded: bool,
    dirty: bool,
    status: String,
    focus_requested: bool,
}

impl DictionaryManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            loaded: false,
            dirty: false,
            status: String::new(),
            focus_requested: false,
        }
    }

    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    pub fn ensure_loaded(&mut self, store: &DictionaryStore) {
        if self.loaded {
            return;
        }
        self.reload(store);
    }

    pub fn reload(&mut self, store: &DictionaryStore) {
        self.entries = store
            .user_entries()
            .into_iter()
            .map(EditableEntry::from_entry)
            .collect();
        self.selected = (!self.entries.is_empty()).then_some(0);
        self.loaded = true;
        self.dirty = false;
        self.status = format!("Loaded {}", store.path().display());
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, store: Arc<DictionaryStore>) {
        self.ensure_loaded(&store);
        let mut close_requested = false;
        let viewport_id = egui::ViewportId::from_hash_of(MANAGER_VIEWPORT_ID);
        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("NPUtella Dictionary")
                .with_inner_size([620.0, 430.0])
                .with_min_inner_size([520.0, 340.0])
                .with_resizable(true)
                .with_decorations(true)
                .with_taskbar(true),
            |ui, _class| {
                apply_manager_style(ui.ctx());
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if ui.input(|i| i.viewport().close_requested()) {
                        close_requested = true;
                    }
                    self.manager_contents(ui, &store, open);
                });
            },
        );
        if self.focus_requested {
            ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Focus);
            self.focus_requested = false;
        }
        if close_requested {
            *open = false;
        }
    }

    fn manager_contents(&mut self, ui: &mut egui::Ui, store: &DictionaryStore, open: &mut bool) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.heading("Dictionary");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    *open = false;
                }
                if ui.button("Reload").clicked() {
                    self.reload(store);
                }
                let save = ui.add_enabled(self.dirty, egui::Button::new("Save"));
                if save.clicked() {
                    self.save(store);
                }
            });
        });
        ui.label(format!(
            "Custom entries saved in {}",
            store.path().display()
        ));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.columns(2, |columns| {
            columns[0].set_width(220.0);
            self.entry_list(&mut columns[0]);
            self.entry_editor(&mut columns[1]);
        });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            if self.dirty {
                ui.colored_label(egui::Color32::from_rgb(230, 175, 75), "Unsaved changes");
            } else {
                ui.label("Saved");
            }
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
        });
    }

    fn entry_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong("Entries");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Add").clicked() {
                    self.entries.push(EditableEntry::new());
                    self.selected = Some(self.entries.len() - 1);
                    self.dirty = true;
                }
            });
        });
        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .id_salt("dictionary_entry_list")
            .max_height(310.0)
            .show(ui, |ui| {
                for (idx, entry) in self.entries.iter().enumerate() {
                    let label = if entry.written.trim().is_empty() {
                        "Untitled"
                    } else {
                        entry.written.trim()
                    };
                    if ui
                        .selectable_label(self.selected == Some(idx), label)
                        .clicked()
                    {
                        self.selected = Some(idx);
                    }
                }
            });
    }

    fn entry_editor(&mut self, ui: &mut egui::Ui) {
        let Some(idx) = self.selected else {
            ui.centered_and_justified(|ui| {
                ui.label("Add an entry to get started.");
            });
            return;
        };
        if idx >= self.entries.len() {
            self.selected = None;
            return;
        }

        let mut remove = false;
        ui.horizontal(|ui| {
            ui.strong("Entry");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Delete").clicked() {
                    remove = true;
                }
            });
        });
        ui.add_space(8.0);

        let entry = &mut self.entries[idx];
        self.dirty |= labeled_text(ui, "Written", &mut entry.written).changed();
        self.dirty |= labeled_text(ui, "Aliases", &mut entry.aliases).changed();
        ui.label("Use commas between aliases, for example: nix os, nicsos, nicks os");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            self.dirty |= ui.checkbox(&mut entry.phonetic, "Phonetic").changed();
            self.dirty |= ui
                .checkbox(&mut entry.high_priority, "High priority")
                .changed();
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Language");
            self.dirty |= ui
                .selectable_value(&mut entry.language, DictionaryLanguage::Any, "Any")
                .changed();
            self.dirty |= ui
                .selectable_value(&mut entry.language, DictionaryLanguage::English, "English")
                .changed();
            self.dirty |= ui
                .selectable_value(&mut entry.language, DictionaryLanguage::French, "French")
                .changed();
        });

        if remove {
            self.entries.remove(idx);
            self.selected = if self.entries.is_empty() {
                None
            } else {
                Some(idx.min(self.entries.len() - 1))
            };
            self.dirty = true;
        }
    }

    fn save(&mut self, store: &DictionaryStore) {
        let entries = self
            .entries
            .iter()
            .map(EditableEntry::to_entry)
            .filter(|entry| !entry.target().trim().is_empty())
            .collect();
        if store.replace_user_entries(entries) {
            self.reload(store);
            self.status = "Saved dictionary".to_string();
        } else {
            self.status = "Save failed; see nputella.log".to_string();
        }
    }
}

impl EditableEntry {
    fn new() -> Self {
        Self {
            written: String::new(),
            aliases: String::new(),
            phonetic: true,
            high_priority: true,
            language: DictionaryLanguage::Any,
        }
    }

    fn from_entry(entry: DictionaryEntry) -> Self {
        let written = entry.target().to_string();
        let mut aliases = entry.aliases;
        if !entry.from.trim().is_empty()
            && !entry.to.trim().is_empty()
            && !aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(entry.from.trim()))
        {
            aliases.insert(0, entry.from.trim().to_string());
        }
        Self {
            written,
            aliases: aliases.join(", "),
            phonetic: entry.phonetic,
            high_priority: entry.priority == DictionaryPriority::High,
            language: entry.language,
        }
    }

    fn to_entry(&self) -> DictionaryEntry {
        let mut entry = DictionaryEntry::new("", self.written.trim());
        entry.aliases = self
            .aliases
            .split(',')
            .map(str::trim)
            .filter(|alias| !alias.is_empty())
            .map(ToString::to_string)
            .collect();
        entry.phonetic = self.phonetic;
        entry.priority = if self.high_priority {
            DictionaryPriority::High
        } else {
            DictionaryPriority::Normal
        };
        entry.language = self.language;
        entry
    }
}

fn labeled_text<'a>(ui: &mut egui::Ui, label: &str, value: &'a mut String) -> egui::Response {
    ui.horizontal(|ui| {
        ui.set_min_height(28.0);
        ui.add_sized([72.0, 22.0], egui::Label::new(label));
        ui.add_sized(
            [ui.available_width(), 24.0],
            egui::TextEdit::singleline(value),
        )
    })
    .inner
}

fn apply_manager_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);
    ctx.set_global_style(style);
}
