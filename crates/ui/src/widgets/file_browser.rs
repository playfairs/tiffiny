use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileBrowser {
    pub id: String,
    pub current_path: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub selected_files: Vec<PathBuf>,
    pub filter: FileFilter,
    pub show_hidden: bool,
    pub multi_select: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(PathBuf) + Send + Sync>>,
    pub on_multi_select: Option<Arc<dyn Fn(Vec<PathBuf>) + Send + Sync>>,
    pub on_navigate: Option<Arc<dyn Fn(PathBuf) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum FileFilter {
    All,
    Audio,
    Image,
    Video,
    Project,
    Custom(Vec<String>),
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

impl FileBrowser {
    pub fn new(id: String) -> Self {
        let current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        
        Self {
            id,
            current_path,
            selected_file: None,
            selected_files: Vec::new(),
            filter: FileFilter::All,
            show_hidden: false,
            multi_select: false,
            enabled: true,
            visible: true,
            on_select: None,
            on_multi_select: None,
            on_navigate: None,
        }
    }

    pub fn current_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_path = path.into();
        self
    }

    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi_select = multi;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(PathBuf) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_multi_select(mut self, callback: impl Fn(Vec<PathBuf>) + Send + Sync + 'static) -> Self {
        self.on_multi_select = Some(Arc::new(callback));
        self
    }

    pub fn on_navigate(mut self, callback: impl Fn(PathBuf) + Send + Sync + 'static) -> Self {
        self.on_navigate = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        ui.horizontal(|ui| {
Navigation bar
            if ui.button("⬅").clicked() {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    changed = true;
                    
                    if let Some(callback) = &self.on_navigate {
                        callback(parent.to_path_buf());
                    }
                }
            }

            ui.add_enabled(self.enabled, egui::TextEdit::singleline(&mut format!("{}", self.current_path.display()))
                .hint_text("Current path")
                .interactive(false));

            if ui.button("🔄").clicked() {
                self.refresh();
                changed = true;
            }

            ui.add_space(10.0);

            egui::ComboBox::from_label("Filter")
                .selected_text(format!("{:?}", self.filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter, FileFilter::All, "All Files");
                    ui.selectable_value(&mut self.filter, FileFilter::Audio, "Audio");
                    ui.selectable_value(&mut self.filter, FileFilter::Image, "Images");
                    ui.selectable_value(&mut self.filter, FileFilter::Video, "Videos");
                    ui.selectable_value(&mut self.filter, FileFilter::Project, "Projects");
                });

            ui.checkbox(&mut self.show_hidden, "Show Hidden");
            ui.checkbox(&mut self.multi_select, "Multi-select");
        });

        ui.separator();

        let entries = self.get_directory_entries();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &entries {
                    if self.render_entry(ui, entry) {
                        changed = true;
                    }
                }
            });

        changed
    }

    fn render_entry(&mut self, ui: &mut egui::Ui, entry: &FileEntry) -> bool {
        let mut clicked = false;
        let mut double_clicked = false;

        ui.horizontal(|ui| {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 24.0),
                egui::Sense::click()
            );

            let is_selected = if self.multi_select {
                self.selected_files.contains(&entry.path)
            } else {
                self.selected_file.as_ref() == Some(&entry.path)
            };

            if is_selected {
                let painter = ui.painter();
                painter.rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
            }

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

            clicked = response.clicked();
            double_clicked = response.double_clicked();

            if clicked {
                if entry.is_directory {
                    self.current_path = entry.path.clone();
                    
                    if let Some(callback) = &self.on_navigate {
                        callback(entry.path.clone());
                    }
                } else {
                    if self.multi_select {
                        if !self.selected_files.contains(&entry.path) {
                            self.selected_files.push(entry.path.clone());
                        } else {
                            self.selected_files.retain(|p| p != &entry.path);
                        }
                        
                        if let Some(callback) = &self.on_multi_select {
                            callback(self.selected_files.clone());
                        }
                    } else {
                        self.selected_file = Some(entry.path.clone());
                        
                        if let Some(callback) = &self.on_select {
                            callback(entry.path.clone());
                        }
                    }
                }
            }

            if double_clicked && entry.is_directory {
                self.current_path = entry.path.clone();
                
                if let Some(callback) = &self.on_navigate {
                    callback(entry.path.clone());
                }
            }
        });

        clicked || double_clicked
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

                if !self.matches_filter(&path, &metadata) {
                    continue;
                }

                let size = metadata.as_ref()
                    .filter(|m| !m.is_dir())
                    .map(|m| m.len())
                    .ok();

                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok());

                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
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

    fn matches_filter(&self, path: &PathBuf, metadata: &Option<std::fs::Metadata>) -> bool {
        match &self.filter {
            FileFilter::All => true,
            FileFilter::Audio => self.is_audio_file(path),
            FileFilter::Image => self.is_image_file(path),
            FileFilter::Video => self.is_video_file(path),
            FileFilter::Project => self.is_project_file(path),
            FileFilter::Custom(extensions) => {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    extensions.contains(&ext.to_lowercase())
                } else {
                    false
                }
            },
        }
    }

    fn is_audio_file(&self, path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), 
                "wav" | "mp3" | "flac" | "ogg" | "aac" | "m4a" | "wma")
        } else {
            false
        }
    }

    fn is_image_file(&self, path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" | "avif")
        } else {
            false
        }
    }

    fn is_video_file(&self, path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(),
                "mp4" | "avi" | "mov" | "mkv" | "webm" | "flv" | "wmv")
        } else {
            false
        }
    }

    fn is_project_file(&self, path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            ext.to_lowercase() == "tiffiny"
        } else {
            false
        }
    }

    fn get_file_icon(&self, extension: &Option<String>) -> bool {
        match extension.as_deref() {
            Some("wav") | Some("mp3") | Some("flac") | Some("ogg") => "🎵",
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "🖼",
            Some("mp4") | Some("avi") | Some("mov") | Some("mkv") => "🎬",
            Some("tiffiny") => "📄",
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

    pub fn refresh(&mut self) {
        self.selected_file = None;
        self.selected_files.clear();
    }

    pub fn navigate_up(&mut self) -> bool {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.refresh();
            
            if let Some(callback) = &self.on_navigate {
                callback(parent.to_path_buf());
            }
            
            true
        } else {
            false
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> bool {
        if path.exists() && path.is_dir() {
            self.current_path = path;
            self.refresh();
            
            if let Some(callback) = &self.on_navigate {
                callback(path);
            }
            
            true
        } else {
            false
        }
    }

    pub fn get_selected_file(&self) -> Option<&PathBuf> {
        self.selected_file.as_ref()
    }

    pub fn get_selected_files(&self) -> &[PathBuf] {
        &self.selected_files
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn clear_selection(&mut self) {
        self.selected_file = None;
        self.selected_files.clear();
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new("default_file_browser".to_string())
    }
}
