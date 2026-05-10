pub mod timeline;
pub mod waveform;
pub mod spectrogram;
pub mod project_explorer;
pub mod inspector;
pub mod console;
pub mod properties;
pub mod effects;
pub mod export;
pub mod help;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct Panel {
    pub id: String,
    pub panel_type: String,
    pub title: String,
    pub visible: bool,
    pub docked: bool,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub z_order: u32,
    pub content: PanelContent,
}

#[derive(Debug, Clone)]
pub enum PanelContent {
    Timeline(timeline::TimelinePanel),
    Waveform(waveform::WaveformPanel),
    Spectrogram(spectrogram::SpectrogramPanel),
    ProjectExplorer(project_explorer::ProjectExplorerPanel),
    Inspector(inspector::InspectorPanel),
    Console(console::ConsolePanel),
    Properties(properties::PropertiesPanel),
    Effects(effects::EffectsPanel),
    Export(export::ExportPanel),
    Help(help::HelpPanel),
}

pub struct PanelManager {
    panels: Arc<RwLock<HashMap<String, Panel>>>,
    visible_panels: Arc<RwLock<Vec<String>>>,
    focused_panel: Arc<RwLock<Option<String>>>,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: Arc::new(RwLock::new(HashMap::new())),
            visible_panels: Arc::new(RwLock::new(Vec::new())),
            focused_panel: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn create_panel(&self, panel: Panel) -> Result<()> {
        let mut panels = self.panels.write();
        panels.insert(panel.id.clone(), panel);
        Ok(())
    }

    pub async fn open_panel(&self, panel_id: &str) {
        let mut visible_panels = self.visible_panels.write();
        if !visible_panels.contains(&panel_id.to_string()) {
            visible_panels.push(panel_id.to_string());
        }

        let mut panels = self.panels.write();
        if let Some(panel) = panels.get_mut(panel_id) {
            panel.visible = true;
        }
    }

    pub async fn close_panel(&self, panel_id: &str) {
        let mut visible_panels = self.visible_panels.write();
        visible_panels.retain(|id| id != panel_id);

        let mut panels = self.panels.write();
        if let Some(panel) = panels.get_mut(panel_id) {
            panel.visible = false;
        }

        let mut focused = self.focused_panel.write();
        if *focused == Some(panel_id.to_string()) {
            *focused = None;
        }
    }

    pub async fn focus_panel(&self, panel_id: &str) {
        let mut focused = self.focused_panel.write();
        *focused = Some(panel_id.to_string());

        let mut panels = self.panels.write();
        if let Some(panel) = panels.get_mut(panel_id) {
            let max_z = panels.values().map(|p| p.z_order).max().unwrap_or(0);
            panel.z_order = max_z + 1;
        }
    }

    pub async def update_panel_position(&self, panel_id: &str, x: i32, y: i32) {
        let mut panels = self.panels.write();
        if let Some(panel) = panels.get_mut(panel_id) {
            panel.position = (x, y);
        }
    }

    pub async fn update_panel_size(&self, panel_id: &str, width: u32, height: u32) {
        let mut panels = self.panels.write();
        if let Some(panel) = panels.get_mut(panel_id) {
            panel.size = (width, height);
        }
    }

    pub fn get_panel(&self, panel_id: &str) -> Option<Panel> {
        let panels = self.panels.read();
        panels.get(panel_id).cloned()
    }

    pub fn get_visible_panels(&self) -> Vec<Panel> {
        let panels = self.panels.read();
        let visible_panels = self.visible_panels.read();
        
        visible_panels.iter()
            .filter_map(|id| panels.get(id).cloned())
            .collect()
    }

    pub fn get_focused_panel(&self) -> Option<Panel> {
        let focused = self.focused_panel.read();
        if let Some(panel_id) = focused.as_ref() {
            let panels = self.panels.read();
            panels.get(panel_id).cloned()
        } else {
            None
        }
    }

    pub async fn update(&self) -> Result<()> {
        let panels = self.panels.read();
        for panel in panels.values() {
            if panel.visible {
                self.update_panel_content(panel).await?;
            }
        }
        Ok(())
    }

    async fn update_panel_content(&self, panel: &Panel) -> Result<()> {
        match &panel.content {
            PanelContent::Timeline(timeline_panel) => {
                timeline_panel.update().await?;
            },
            PanelContent::Waveform(waveform_panel) => {
                waveform_panel.update().await?;
            },
            PanelContent::Spectrogram(spectrogram_panel) => {
                spectrogram_panel.update().await?;
            },
            PanelContent::ProjectExplorer(project_explorer_panel) => {
                project_explorer_panel.update().await?;
            },
            PanelContent::Inspector(inspector_panel) => {
                inspector_panel.update().await?;
            },
            PanelContent::Console(console_panel) => {
                console_panel.update().await?;
            },
            PanelContent::Properties(properties_panel) => {
                properties_panel.update().await?;
            },
            PanelContent::Effects(effects_panel) => {
                effects_panel.update().await?;
            },
            PanelContent::Export(export_panel) => {
                export_panel.update().await?;
            },
            PanelContent::Help(help_panel) => {
                help_panel.update().await?;
            },
        }
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        let panels = self.panels.read();
        for panel in panels.values() {
            match &panel.content {
                PanelContent::Timeline(timeline_panel) => {
                    timeline_panel.cleanup().await?;
                },
                PanelContent::Waveform(waveform_panel) => {
                    waveform_panel.cleanup().await?;
                },
                PanelContent::Spectrogram(spectrogram_panel) => {
                    spectrogram_panel.cleanup().await?;
                },
                PanelContent::ProjectExplorer(project_explorer_panel) => {
                    project_explorer_panel.cleanup().await?;
                },
                PanelContent::Inspector(inspector_panel) => {
                    inspector_panel.cleanup().await?;
                },
                PanelContent::Console(console_panel) => {
                    console_panel.cleanup().await?;
                },
                PanelContent::Properties(properties_panel) => {
                    properties_panel.cleanup().await?;
                },
                PanelContent::Effects(effects_panel) => {
                    effects_panel.cleanup().await?;
                },
                PanelContent::Export(export_panel) => {
                    export_panel.cleanup().await?;
                },
                PanelContent::Help(help_panel) => {
                    help_panel.cleanup().await?;
                },
            }
        }
        Ok(())
    }

    pub fn create_default_panels(&self) -> Vec<Panel> {
        vec![
            Panel {
                id: "timeline".to_string(),
                panel_type: "timeline".to_string(),
                title: "Timeline".to_string(),
                visible: true,
                docked: true,
                position: (0, 800),
                size: (1920, 200),
                z_order: 1,
                content: PanelContent::Timeline(timeline::TimelinePanel::new()),
            },
            Panel {
                id: "project_explorer".to_string(),
                panel_type: "project_explorer".to_string(),
                title: "Project Explorer".to_string(),
                visible: true,
                docked: true,
                position: (0, 28),
                size: (280, 772),
                z_order: 2,
                content: PanelContent::ProjectExplorer(project_explorer::ProjectExplorerPanel::new()),
            },
            Panel {
                id: "inspector".to_string(),
                panel_type: "inspector".to_string(),
                title: "Inspector".to_string(),
                visible: true,
                docked: true,
                position: (1640, 28),
                size: (280, 772),
                z_order: 3,
                content: PanelContent::Inspector(inspector::InspectorPanel::new()),
            },
            Panel {
                id: "waveform".to_string(),
                panel_type: "waveform".to_string(),
                title: "Waveform".to_string(),
                visible: false,
                docked: true,
                position: (280, 28),
                size: (400, 300),
                z_order: 4,
                content: PanelContent::Waveform(waveform::WaveformPanel::new()),
            },
            Panel {
                id: "spectrogram".to_string(),
                panel_type: "spectrogram".to_string(),
                title: "Spectrogram".to_string(),
                visible: false,
                docked: true,
                position: (680, 28),
                size: (400, 300),
                z_order: 5,
                content: PanelContent::Spectrogram(spectrogram::SpectrogramPanel::new()),
            },
            Panel {
                id: "console".to_string(),
                panel_type: "console".to_string(),
                title: "Console".to_string(),
                visible: false,
                docked: true,
                position: (280, 328),
                size: (680, 200),
                z_order: 6,
                content: PanelContent::Console(console::ConsolePanel::new()),
            },
        ]
    }
}
