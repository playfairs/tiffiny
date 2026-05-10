use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use tiffiny_core::prelude::*;

#[derive(Debug, Clone)]
pub struct ProjectExplorerPanel {
    pub project_id: Option<Uuid>,
    pub assets: Vec<AssetNode>,
    pub selected_assets: Vec<String>,
    pub expanded_nodes: std::collections::HashSet<String>,
    pub search_query: String,
    pub filter_type: Option<AssetType>,
    pub sort_by: SortBy,
    pub sort_ascending: bool,
    pub show_thumbnails: bool,
}

#[derive(Debug, Clone)]
pub struct AssetNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub asset_type: Option<AssetType>,
    pub file_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_seconds: Option<f64>,
    pub thumbnail_path: Option<String>,
    pub children: Vec<AssetNode>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Root,
    Folder,
    Asset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Audio,
    Image,
    Video,
    Raw,
    Project,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Type,
    Size,
    Duration,
    Modified,
}

impl ProjectExplorerPanel {
    pub fn new() -> Self {
        Self {
            project_id: None,
            assets: Vec::new(),
            selected_assets: Vec::new(),
            expanded_nodes: std::collections::HashSet::new(),
            search_query: String::new(),
            filter_type: None,
            sort_by: SortBy::Name,
            sort_ascending: true,
            show_thumbnails: true,
        }
    }

    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn set_project(&mut self, project_id: Uuid) {
        self.project_id = Some(project_id);
        self.load_project_assets();
    }

    pub fn clear_project(&mut self) {
        self.project_id = None;
        self.assets.clear();
        self.selected_assets.clear();
        self.expanded_nodes.clear();
    }

    fn load_project_assets(&mut self) {
        self.assets = vec![
            AssetNode {
                id: "audio_folder".to_string(),
                name: "Audio".to_string(),
                node_type: NodeType::Folder,
                asset_type: Some(AssetType::Audio),
                file_path: None,
                size_bytes: None,
                duration_seconds: None,
                thumbnail_path: None,
                children: vec![
                    AssetNode {
                        id: "audio_1".to_string(),
                        name: "sample.wav".to_string(),
                        node_type: NodeType::Asset,
                        asset_type: Some(AssetType::Audio),
                        file_path: Some("/path/to/sample.wav".to_string()),
                        size_bytes: Some(1024 * 1024),
                        duration_seconds: Some(180.0),
                        thumbnail_path: None,
                        children: Vec::new(),
                        parent_id: Some("audio_folder".to_string()),
                    },
                ],
                parent_id: None,
            },
            AssetNode {
                id: "image_folder".to_string(),
                name: "Images".to_string(),
                node_type: NodeType::Folder,
                asset_type: Some(AssetType::Image),
                file_path: None,
                size_bytes: None,
                duration_seconds: None,
                thumbnail_path: None,
                children: vec![
                    AssetNode {
                        id: "image_1".to_string(),
                        name: "texture.png".to_string(),
                        node_type: NodeType::Asset,
                        asset_type: Some(AssetType::Image),
                        file_path: Some("/path/to/texture.png".to_string()),
                        size_bytes: Some(512 * 512),
                        duration_seconds: None,
                        thumbnail_path: Some("/path/to/thumbnail.png".to_string()),
                        children: Vec::new(),
                        parent_id: Some("image_folder".to_string()),
                    },
                ],
                parent_id: None,
            },
            AssetNode {
                id: "video_folder".to_string(),
                name: "Video".to_string(),
                node_type: NodeType::Folder,
                asset_type: Some(AssetType::Video),
                file_path: None,
                size_bytes: None,
                duration_seconds: None,
                thumbnail_path: None,
                children: vec![
                    AssetNode {
                        id: "video_1".to_string(),
                        name: "clip.mp4".to_string(),
                        node_type: NodeType::Asset,
                        asset_type: Some(AssetType::Video),
                        file_path: Some("/path/to/clip.mp4".to_string()),
                        size_bytes: Some(10 * 1024 * 1024),
                        duration_seconds: Some(300.0),
                        thumbnail_path: Some("/path/to/video_thumb.jpg".to_string()),
                        children: Vec::new(),
                        parent_id: Some("video_folder".to_string()),
                    },
                ],
                parent_id: None,
            },
        ];

        self.expanded_nodes.insert("audio_folder".to_string());
        self.expanded_nodes.insert("image_folder".to_string());
        self.expanded_nodes.insert("video_folder".to_string());
    }

    pub fn add_asset(&mut self, asset: AssetNode) {
        self.assets.push(asset);
    }

    pub fn remove_asset(&mut self, asset_id: &str) -> Option<AssetNode> {
        let index = self.assets.iter().position(|a| a.id == asset_id);
        if let Some(index) = index {
            Some(self.assets.remove(index))
        } else {
            None
        }
    }

    pub fn get_asset(&self, asset_id: &str) -> Option<&AssetNode> {
        self.assets.iter().find(|a| a.id == asset_id)
    }

    pub fn toggle_selection(&mut self, asset_id: &str) {
        if self.selected_assets.contains(&asset_id.to_string()) {
            self.selected_assets.retain(|id| id != asset_id);
        } else {
            self.selected_assets.push(asset_id.to_string());
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_assets.clear();
    }

    pub fn select_all(&mut self) {
        self.selected_assets.clear();
        for asset in &self.assets {
            if asset.node_type == NodeType::Asset {
                self.selected_assets.push(asset.id.clone());
            }
        }
    }

    pub fn toggle_expansion(&mut self, node_id: &str) {
        if self.expanded_nodes.contains(node_id) {
            self.expanded_nodes.remove(node_id);
        } else {
            self.expanded_nodes.insert(node_id.to_string());
        }
    }

    pub fn expand_all(&mut self) {
        for asset in &self.assets {
            if asset.node_type == NodeType::Folder {
                self.expanded_nodes.insert(asset.id.clone());
            }
        }
    }

    pub fn collapse_all(&mut self) {
        self.expanded_nodes.clear();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn set_filter_type(&mut self, filter_type: Option<AssetType>) {
        self.filter_type = filter_type;
    }

    pub fn set_sort_by(&mut self, sort_by: SortBy) {
        self.sort_by = sort_by;
    }

    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
    }

    pub fn toggle_thumbnails(&mut self) {
        self.show_thumbnails = !self.show_thumbnails;
    }

    fn filter_and_sort_assets(&self) -> Vec<&AssetNode> {
        let mut filtered: Vec<&AssetNode> = self.assets
            .iter()
            .filter(|asset| {
                if !self.search_query.is_empty() {
                    !asset.name.to_lowercase().contains(&self.search_query.to_lowercase())
                } else {
                    true
                }
            })
            .filter(|asset| {
                if let Some(filter_type) = &self.filter_type {
                    asset.asset_type.as_ref().map_or(false, |t| t == filter_type)
                } else {
                    true
                }
            })
            .collect();

        filtered.sort_by(|a, b| {
            let comparison = match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Type => a.asset_type.cmp(&b.asset_type),
                SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
                SortBy::Duration => a.duration_seconds.cmp(&b.duration_seconds),
                SortBy::Modified => std::cmp::Ordering::Equal,
            };

            if self.sort_ascending {
                comparison
            } else {
                comparison.reverse()
            }
        });

        filtered
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("Search assets..."));

            if ui.button("🔍").clicked() {
            }

            ui.add_space(10.0);

            egui::ComboBox::from_label("Filter")
                .selected_text(format!("{:?}", self.filter_type.unwrap_or(AssetType::Audio)))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_type, None, "All");
                    ui.selectable_value(&mut self.filter_type, Some(AssetType::Audio), "Audio");
                    ui.selectable_value(&mut self.filter_type, Some(AssetType::Image), "Images");
                    ui.selectable_value(&mut self.filter_type, Some(AssetType::Video), "Video");
                    ui.selectable_value(&mut self.filter_type, Some(AssetType::Raw), "Raw");
                });

            ui.add_space(10.0);

            egui::ComboBox::from_label("Sort")
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, SortBy::Name, "Name");
                    ui.selectable_value(&mut self.sort_by, SortBy::Type, "Type");
                    ui.selectable_value(&mut self.sort_by, SortBy::Size, "Size");
                    ui.selectable_value(&mut self.sort_by, SortBy::Duration, "Duration");
                    ui.selectable_value(&mut self.sort_by, SortBy::Modified, "Modified");
                });

            if ui.button(if self.sort_ascending { "↑" } else { "↓" }).clicked() {
                self.toggle_sort_direction();
            }

            ui.add_space(10.0);

            ui.checkbox(&mut self.show_thumbnails, "Thumbnails");

            ui.add_space(10.0);

            if ui.button("Expand All").clicked() {
                self.expand_all();
            }

            if ui.button("Collapse All").clicked() {
                self.collapse_all();
            }
        });

        ui.separator();

        if let Some(project_id) = self.project_id {
            ui.label(format!("Project: {}", project_id));
        } else {
            ui.label("No project loaded");
        }

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.show_thumbnails {
                    self.render_thumbnail_view(ui);
                } else {
                    self.render_tree_view(ui);
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(format!("{} assets selected", self.selected_assets.len()));

            if !self.selected_assets.is_empty() {
                if ui.button("Import").clicked() {
                    self.import_selected_assets();
                }

                if ui.button("Delete").clicked() {
                    self.delete_selected_assets();
                }

                if ui.button("Properties").clicked() {
                    self.show_asset_properties();
                }
            }
        });
    }

    fn render_tree_view(&mut self, ui: &mut egui::Ui) {
        let filtered_assets = self.filter_and_sort_assets();
        
        for asset in filtered_assets {
            self.render_tree_node(ui, asset, 0);
        }
    }

    fn render_tree_node(&mut self, ui: &mut egui::Ui, asset: &AssetNode, depth: usize) {
        ui.horizontal(|ui| {
            let indent = depth as f32 * 20.0;
            ui.add_space(indent);

            if asset.node_type == NodeType::Folder {
                let is_expanded = self.expanded_nodes.contains(&asset.id);
                let icon = if is_expanded { "▼" } else { "▶" };
                
                if ui.button(icon).clicked() {
                    self.toggle_expansion(&asset.id);
                }
            } else {
                ui.add_space(16.0);
            }

            let is_selected = self.selected_assets.contains(&asset.id);
            let label_color = if is_selected {
                egui::Color32::from_rgb(100, 150, 255)
            } else {
                egui::Color32::WHITE
            };

            ui.colored_label(label_color, &asset.name);

            if ui.button("👁").clicked() {
                self.preview_asset(asset);
            }

            if ui.button("📋").clicked() {
                self.copy_asset_path(asset);
            }
        });

        if asset.node_type == NodeType::Folder && self.expanded_nodes.contains(&asset.id) {
            for child in &asset.children {
                self.render_tree_node(ui, child, depth + 1);
            }
        }
    }

    fn render_thumbnail_view(&mut self, ui: &mut egui::Ui) {
        let filtered_assets = self.filter_and_sort_assets();
        let thumbnail_size = 120.0;
        let padding = 10.0;
        let items_per_row = ((ui.available_width() + padding) / (thumbnail_size + padding)) as usize;

        for (index, asset) in filtered_assets.iter().enumerate() {
            if index % items_per_row == 0 {
                ui.horizontal(|ui| {
                    for i in 0..items_per_row.min(filtered_assets.len() - index) {
                        if let Some(current_asset) = filtered_assets.get(index + i) {
                            self.render_thumbnail_item(ui, current_asset, thumbnail_size);
                        }
                    }
                });
            }
        }
    }

    fn render_thumbnail_item(&mut self, ui: &mut egui::Ui, asset: &AssetNode, size: f32) {
        ui.vertical(|ui| {
            let is_selected = self.selected_assets.contains(&asset.id);
            let frame_color = if is_selected {
                egui::Color32::from_rgb(100, 150, 255)
            } else {
                egui::Color32::from_rgb(64, 64, 64)
            };

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(size, size + 30.0),
                egui::Sense::click()
            );

            let painter = ui.painter();
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(2.0, frame_color)
            );

            if let Some(thumbnail_path) = &asset.thumbnail_path {
                painter.rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(size, size)),
                    4.0,
                    egui::Color32::from_rgb(80, 80, 80)
                );
            } else {
                let icon = match asset.asset_type {
                    Some(AssetType::Audio) => "🎵",
                    Some(AssetType::Image) => "🖼",
                    Some(AssetType::Video) => "🎬",
                    Some(AssetType::Raw) => "📄",
                    _ => "📁",
                };

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE
                );
            }

            painter.text(
                egui::pos2(rect.min.x, rect.max.y - 20.0),
                egui::Align2::LEFT_BOTTOM,
                &asset.name,
                egui::FontId::default(),
                egui::Color32::WHITE
            );

            if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
                self.toggle_selection(&asset.id);
            }
        });
    }

    fn import_selected_assets(&self) {
        tracing::info!("Importing {} selected assets", self.selected_assets.len());
    }

    fn delete_selected_assets(&mut self) {
        tracing::info!("Deleting {} selected assets", self.selected_assets.len());
        for asset_id in &self.selected_assets.clone() {
            self.remove_asset(asset_id);
        }
        self.clear_selection();
    }

    fn show_asset_properties(&self) {
        tracing::info!("Showing properties for {} selected assets", self.selected_assets.len());
    }

    fn preview_asset(&self, asset: &AssetNode) {
        tracing::info!("Previewing asset: {}", asset.name);
    }

    fn copy_asset_path(&self, asset: &AssetNode) {
        if let Some(file_path) = &asset.file_path {
            tracing::info!("Copied asset path: {}", file_path);
        }
    }
}

impl Default for ProjectExplorerPanel {
    fn default() -> Self {
        Self::new()
    }
}
