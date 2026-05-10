use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub version: String,
    pub window_size: (u32, u32),
    pub panels: HashMap<String, PanelLayout>,
    pub dock_areas: Vec<DockArea>,
    pub menu_bar: MenuBarLayout,
    pub status_bar: StatusBarLayout,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    pub panel_id: String,
    pub panel_type: String,
    pub visible: bool,
    pub docked: bool,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub z_order: u32,
    pub constraints: PanelConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConstraints {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub resizable: bool,
    pub movable: bool,
    pub closable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockArea {
    pub id: String,
    pub orientation: DockOrientation,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub panels: Vec<DockedPanel>,
    pub splitter_positions: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockedPanel {
    pub panel_id: String,
    pub size_ratio: f32,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuBarLayout {
    pub visible: bool,
    pub height: u32,
    pub items: Vec<MenuItemLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemLayout {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub separator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarLayout {
    pub visible: bool,
    pub height: u32,
    pub sections: Vec<StatusSectionLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSectionLayout {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub alignment: StatusAlignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusAlignment {
    Left,
    Center,
    Right,
}

impl Layout {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: "1.0.0".to_string(),
            window_size: (1920, 1080),
            panels: HashMap::new(),
            dock_areas: Vec::new(),
            menu_bar: MenuBarLayout::default(),
            status_bar: StatusBarLayout::default(),
            theme: "dark".to_string(),
        }
    }

    pub fn load_default_preset(&mut self) {
        self.add_default_panels();
        self.create_default_dock_areas();
        self.setup_default_menu_bar();
        self.setup_default_status_bar();
    }

    fn add_default_panels(&mut self) {
        let default_panels = vec![
            PanelLayout {
                panel_id: "timeline".to_string(),
                panel_type: "timeline".to_string(),
                visible: true,
                docked: true,
                position: (0, 800),
                size: (1920, 200),
                z_order: 1,
                constraints: PanelConstraints {
                    min_width: 400,
                    min_height: 100,
                    max_width: None,
                    max_height: None,
                    resizable: true,
                    movable: false,
                    closable: false,
                },
            },
            PanelLayout {
                panel_id: "project_explorer".to_string(),
                panel_type: "project_explorer".to_string(),
                visible: true,
                docked: true,
                position: (0, 28),
                size: (280, 772),
                z_order: 2,
                constraints: PanelConstraints {
                    min_width: 200,
                    min_height: 200,
                    max_width: Some(400),
                    max_height: None,
                    resizable: true,
                    movable: true,
                    closable: true,
                },
            },
            PanelLayout {
                panel_id: "inspector".to_string(),
                panel_type: "inspector".to_string(),
                visible: true,
                docked: true,
                position: (1640, 28),
                size: (280, 772),
                z_order: 3,
                constraints: PanelConstraints {
                    min_width: 200,
                    min_height: 200,
                    max_width: Some(400),
                    max_height: None,
                    resizable: true,
                    movable: true,
                    closable: true,
                },
            },
            PanelLayout {
                panel_id: "waveform".to_string(),
                panel_type: "waveform".to_string(),
                visible: false,
                docked: true,
                position: (280, 28),
                size: (400, 300),
                z_order: 4,
                constraints: PanelConstraints {
                    min_width: 300,
                    min_height: 150,
                    max_width: None,
                    max_height: None,
                    resizable: true,
                    movable: true,
                    closable: true,
                },
            },
            PanelLayout {
                panel_id: "spectrogram".to_string(),
                panel_type: "spectrogram".to_string(),
                visible: false,
                docked: true,
                position: (680, 28),
                size: (400, 300),
                z_order: 5,
                constraints: PanelConstraints {
                    min_width: 300,
                    min_height: 150,
                    max_width: None,
                    max_height: None,
                    resizable: true,
                    movable: true,
                    closable: true,
                },
            },
            PanelLayout {
                panel_id: "console".to_string(),
                panel_type: "console".to_string(),
                visible: false,
                docked: true,
                position: (280, 328),
                size: (680, 200),
                z_order: 6,
                constraints: PanelConstraints {
                    min_width: 400,
                    min_height: 100,
                    max_width: None,
                    max_height: None,
                    resizable: true,
                    movable: true,
                    closable: true,
                },
            },
        ];

        for panel in default_panels {
            self.panels.insert(panel.panel_id.clone(), panel);
        }
    }

    fn create_default_dock_areas(&mut self) {
        self.dock_areas = vec![
            DockArea {
                id: "main_dock".to_string(),
                orientation: DockOrientation::Horizontal,
                position: (280, 28),
                size: (1360, 772),
                panels: vec![
                    DockedPanel {
                        panel_id: "viewport".to_string(),
                        size_ratio: 0.6,
                        visible: true,
                    },
                    DockedPanel {
                        panel_id: "right_dock".to_string(),
                        size_ratio: 0.4,
                        visible: true,
                    },
                ],
                splitter_positions: vec![0.6],
            },
            DockArea {
                id: "right_dock".to_string(),
                orientation: DockOrientation::Vertical,
                position: (1104, 28),
                size: (280, 772),
                panels: vec![
                    DockedPanel {
                        panel_id: "inspector".to_string(),
                        size_ratio: 0.5,
                        visible: true,
                    },
                    DockedPanel {
                        panel_id: "console".to_string(),
                        size_ratio: 0.5,
                        visible: false,
                    },
                ],
                splitter_positions: vec![0.5],
            },
        ];
    }

    fn setup_default_menu_bar(&mut self) {
        self.menu_bar = MenuBarLayout {
            visible: true,
            height: 28,
            items: vec![
                MenuItemLayout {
                    id: "file".to_string(),
                    label: "File".to_string(),
                    shortcut: None,
                    enabled: true,
                    separator: false,
                },
                MenuItemLayout {
                    id: "edit".to_string(),
                    label: "Edit".to_string(),
                    shortcut: None,
                    enabled: true,
                    separator: false,
                },
                MenuItemLayout {
                    id: "view".to_string(),
                    label: "View".to_string(),
                    shortcut: None,
                    enabled: true,
                    separator: false,
                },
                MenuItemLayout {
                    id: "effects".to_string(),
                    label: "Effects".to_string(),
                    shortcut: None,
                    enabled: true,
                    separator: false,
                },
                MenuItemLayout {
                    id: "help".to_string(),
                    label: "Help".to_string(),
                    shortcut: None,
                    enabled: true,
                    separator: false,
                },
            ],
        };
    }

    fn setup_default_status_bar(&mut self) {
        self.status_bar = StatusBarLayout {
            visible: true,
            height: 24,
            sections: vec![
                StatusSectionLayout {
                    id: "status".to_string(),
                    label: "Ready".to_string(),
                    width: 200,
                    alignment: StatusAlignment::Left,
                },
                StatusSectionLayout {
                    id: "project_info".to_string(),
                    label: "No project".to_string(),
                    width: 300,
                    alignment: StatusAlignment::Left,
                },
                StatusSectionLayout {
                    id: "performance".to_string(),
                    label: "60 FPS".to_string(),
                    width: 100,
                    alignment: StatusAlignment::Right,
                },
            ],
        };
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = (width, height);
        self.adjust_panel_positions();
    }

    fn adjust_panel_positions(&mut self) {
        let (window_width, window_height) = self.window_size;
        let menu_height = if self.menu_bar.visible { self.menu_bar.height } else { 0 };
        let status_height = if self.status_bar.visible { self.status_bar.height } else { 0 };

        let available_height = window_height.saturating_sub(menu_height + status_height);

        for panel in self.panels.values_mut() {
            if panel.docked {
                match panel.panel_id.as_str() {
                    "timeline" => {
                        panel.position = (0, (available_height - panel.size.1) as i32);
                        panel.size.0 = window_width;
                    },
                    "project_explorer" => {
                        panel.position = (0, menu_height as i32);
                        panel.size.1 = available_height;
                    },
                    "inspector" => {
                        panel.position = ((window_width - panel.size.0) as i32, menu_height as i32);
                        panel.size.1 = available_height;
                    },
                    _ => {}
                }
            }
        }

        for dock_area in &mut self.dock_areas {
            match dock_area.id.as_str() {
                "main_dock" => {
                    let explorer_width = self.panels.get("project_explorer")
                        .map(|p| p.size.0)
                        .unwrap_or(280);
                    let inspector_width = self.panels.get("inspector")
                        .map(|p| p.size.0)
                        .unwrap_or(280);
                    
                    dock_area.position = (explorer_width as i32, menu_height as i32);
                    dock_area.size = (
                        window_width - explorer_width - inspector_width,
                        available_height
                    );
                },
                _ => {}
            }
        }
    }

    pub fn get_panel(&self, panel_id: &str) -> Option<&PanelLayout> {
        self.panels.get(panel_id)
    }

    pub fn get_mut_panel(&mut self, panel_id: &str) -> Option<&mut PanelLayout> {
        self.panels.get_mut(panel_id)
    }

    pub fn add_panel(&mut self, panel: PanelLayout) {
        self.panels.insert(panel.panel_id.clone(), panel);
    }

    pub fn remove_panel(&mut self, panel_id: &str) -> Option<PanelLayout> {
        self.panels.remove(panel_id)
    }

    pub fn get_main_area_rect(&self) -> egui::Rect {
        let (window_width, window_height) = self.window_size;
        let menu_height = if self.menu_bar.visible { self.menu_bar.height } else { 0 };
        let status_height = if self.status_bar.visible { self.status_bar.height } else { 0 };
        let timeline_height = self.panels.get("timeline")
            .filter(|p| p.visible)
            .map(|p| p.size.1)
            .unwrap_or(0);

        let explorer_width = self.panels.get("project_explorer")
            .filter(|p| p.visible)
            .map(|p| p.size.0)
            .unwrap_or(0);

        let inspector_width = self.panels.get("inspector")
            .filter(|p| p.visible)
            .map(|p| p.size.0)
            .unwrap_or(0);

        let x = explorer_width as f32;
        let y = menu_height as f32;
        let width = (window_width - explorer_width - inspector_width) as f32;
        let height = (window_height - menu_height - status_height - timeline_height) as f32;

        egui::Rect::from_min_max(
            egui::pos2(x, y),
            egui::pos2(x + width, y + height)
        )
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let layout: Self = serde_json::from_str(&json)?;
        Ok(layout)
    }
}

impl Default for Layout {
    fn default() -> Self {
        let mut layout = Self::new("Default".to_string());
        layout.load_default_preset();
        layout
    }
}

impl Default for MenuBarLayout {
    fn default() -> Self {
        Self {
            visible: true,
            height: 28,
            items: Vec::new(),
        }
    }
}

impl Default for StatusBarLayout {
    fn default() -> Self {
        Self {
            visible: true,
            height: 24,
            sections: Vec::new(),
        }
    }
}

impl Default for PanelConstraints {
    fn default() -> Self {
        Self {
            min_width: 100,
            min_height: 100,
            max_width: None,
            max_height: None,
            resizable: true,
            movable: true,
            closable: true,
        }
    }
}
