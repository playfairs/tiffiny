use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileDialog {
    pub id: String,
    pub title: String,
    pub mode: FileDialogMode,
    pub initial_path: Option<PathBuf>,
    pub file_types: Vec<FileType>,
    pub multi_select: bool,
    pub show_hidden: bool,
    pub selected_files: Vec<PathBuf>,
    pub current_path: PathBuf,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(Vec<PathBuf>) + Send + Sync>>,
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileDialogMode {
    OpenFile,
    SaveFile,
    SelectFolder,
    SelectFiles,
}

#[derive(Debug, Clone)]
pub struct FileType {
    pub name: String,
    pub extensions: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
    pub extension: Option<String>,
    pub icon: String,
}

impl FileDialog {
    pub fn new(id: String) -> Self {
        let initial_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        
        Self {
            id,
            title: "File Dialog".to_string(),
            mode: FileDialogMode::OpenFile,
            initial_path: Some(initial_path.clone()),
            file_types: vec![
                FileType {
                    name: "All Files".to_string(),
                    extensions: vec!["*".to_string()],
                    description: "All files (*.*)".to_string(),
                },
            ],
            multi_select: false,
            show_hidden: false,
            selected_files: Vec::new(),
            current_path: initial_path,
            visible: false,
            on_select: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn mode(mut self, mode: FileDialogMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn initial_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.initial_path = Some(path.into());
        self
    }

    pub fn file_types(mut self, file_types: Vec<FileType>) -> Self {
        self.file_types = file_types;
        self
    }

    pub fn add_file_type(mut self, file_type: FileType) -> Self {
        self.file_types.push(file_type);
        self
    }

    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi_select = multi;
        self
    }

    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(Vec<PathBuf>) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(callback));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        if let Some(path) = &self.initial_path {
            self.current_path = path.clone();
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.selected_files.clear();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_selected_files(&self) -> &[PathBuf] {
        &self.selected_files
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn set_current_path(&mut self, path: PathBuf) {
        self.current_path = path;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(600.0, 400.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
                    .rounding(8.0);

                frame.show(ui, |ui| {
                    self.render_title_bar(ui, &mut closed);
                    ui.separator();
                    self.render_content(ui, &mut closed);
                    self.render_buttons(ui, &mut closed);
                });
            });

        if closed {
            self.hide();
            if let Some(callback) = &self.on_cancel {
                callback();
            }
        }

        closed
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
File browser
            let available_width = ui.available_width() * 0.6;
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(available_width, 300.0),
                egui::Sense::hover()
            );

            let painter = ui.painter();
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(40, 40, 40));
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)));

            let entries = self.get_directory_entries();
            let mut clip_rect = egui::Rect::from_min_size(rect.min, egui::vec2(available_width - 4.0, 296.0));
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &entries {
                        if self.render_file_entry(ui, entry, &mut clip_rect) {
                            *closed = true;
                            if self.multi_select {
                            } else {
                                self.selected_files.clear();
                                self.selected_files.push(entry.path.clone());
                            }
                            
                            if let Some(callback) = &self.on_select {
                                callback(self.selected_files.clone());
                            }
                        }
                    }
                });
            });

            ui.vertical(|ui| {
                ui.label("Current Path:");
                let mut path_string = self.current_path.to_string_lossy().to_string();
                if ui.add(egui::TextEdit::singleline(&mut path_string)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter path...")
                    .changed() {
                } {
                }

                ui.horizontal(|ui| {
                    if ui.button("⬅").clicked() {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                        }
                    }

                    if ui.button("🏠").clicked() {
                        if let Some(home) = dirs::home_dir() {
                            self.current_path = home;
                        }
                    }

                    if ui.button("🔄").clicked() {
                    }
                });

                ui.separator();

                ui.label("File Types:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.file_types[0].name)
                    .show_ui(ui, |ui| {
                        for file_type in &self.file_types {
                            ui.selectable_value(&mut self.file_types[0], file_type, &file_type.name);
                        }
                    });

                ui.separator();

                if let Some(selected_type) = self.file_types.first() {
                    ui.label(&selected_type.description);
                    ui.label("Extensions:");
                    for ext in &selected_type.extensions {
                        ui.label(format!("*.{}", ext));
                    }
                }
            });
        });
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            let button_text = match self.mode {
                FileDialogMode::OpenFile => "Open",
                FileDialogMode::SaveFile => "Save",
                FileDialogMode::SelectFolder => "Select",
                FileDialogMode::SelectFiles => "Select",
            };

            if ui.button(button_text).clicked() {
                if !self.selected_files.is_empty() {
                } else {
                    *closed = true;
                    if let Some(callback) = &self.on_select {
                        callback(self.selected_files.clone());
                    }
                }
            }

            ui.add_space(10.0);

            if ui.button("Cancel").clicked() {
                *closed = true;
            }
        });
    }

    fn render_file_entry(&self, ui: &mut egui::Ui, entry: &FileEntry, clip_rect: &mut egui::Rect) -> bool {
        let mut clicked = false;

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 24.0),
            egui::Sense::click()
        );

        if rect.min.y >= clip_rect.min.y && rect.max.y <= clip_rect.max.y {
            let is_selected = self.selected_files.contains(&entry.path);
            
            if is_selected {
                let painter = ui.painter();
                painter.rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
            }

            ui.horizontal(|ui| {
                ui.label(&entry.icon);

                let name_color = if entry.is_directory {
                    egui::Color32::from_rgb(100, 150, 255)
                } else {
                    egui::Color32::WHITE
                };

                ui.colored_label(name_color, &entry.name);

                if let Some(size) = entry.size {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(self.format_file_size(size));
                    });
                }
            });

            if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
                clicked = true;
            }
        }

        clicked
    }

    fn get_directory_entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        if let Ok(dir_entries) = std::fs::read_dir(&self.current_path) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }

                let metadata = entry.metadata().ok();
                let is_directory = metadata.as_ref()
                    .map(|m| m.is_dir())
                    .unwrap_or(false);

                let size = metadata.as_ref()
                    .filter(|m| !m.is_dir())
                    .map(|m| m.len())
                    .ok();

                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok());

                let extension = path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());

                let icon = if is_directory {
                    "📁".to_string()
                } else {
                    self.get_file_icon(&extension)
                };

                entries.push(FileEntry {
                    path,
                    name,
                    is_directory,
                    size,
                    modified,
                    extension,
                    icon,
                });
            }
        }

        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, true) | (false, false) => {
                    a.name.cmp(&b.name)
                },
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
            }
        });

        entries
    }

    fn get_file_icon(&self, extension: &Option<String>) -> String {
        match extension.as_deref() {
            Some("txt") | Some("md") | Some("rst") => "📄",
            Some("pdf") => "📕",
            Some("doc") | Some("docx") => "📘",
            Some("xls") | Some("xlsx") => "📗",
            Some("ppt") | Some("pptx") => "📙",
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp") => "🖼",
            Some("mp3") | Some("wav") | Some("flac") | Some("ogg") | Some("aac") => "🎵",
            Some("mp4") | Some("avi") | Some("mov") | Some("mkv") | Some("webm") => "🎬",
            Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => "🗜",
            Some("exe") | Some("msi") | Some("deb") | Some("rpm") | Some("dmg") | Some("pkg") => "⚙",
            _ => "📄",
        }.to_string()
    }

    fn format_file_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    pub fn set_selected_files(&mut self, files: Vec<PathBuf>) {
        self.selected_files = files;
    }

    pub fn clear_selection(&mut self) {
        self.selected_files.clear();
    }

    pub fn navigate_up(&mut self) -> bool {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            true
        } else {
            false
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> bool {
        if path.exists() && (path.is_dir() || matches!(self.mode, FileDialogMode::SaveFile)) {
            self.current_path = path;
            true
        } else {
            false
        }
    }
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new("default_file_dialog".to_string())
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: "All Files".to_string(),
            extensions: vec!["*".to_string()],
            description: "All files (*.*)".to_string(),
        }
    }
}
