use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct DockingManager {
    pub dock_areas: Arc<RwLock<Vec<DockArea>>>,
    pub panels: Arc<RwLock<std::collections::HashMap<String, DockPanel>>>,
    pub drag_state: Arc<RwLock<DragState>>,
}

#[derive(Debug, Clone)]
pub struct DockArea {
    pub id: String,
    pub orientation: DockOrientation,
    pub position: egui::Rect,
    pub panels: Vec<DockedPanel>,
    pub splitter_positions: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DockOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct DockedPanel {
    pub panel_id: String,
    pub size_ratio: f32,
    pub visible: bool,
    pub min_size: egui::Vec2,
    pub max_size: Option<egui::Vec2>,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub dragging: bool,
    pub dragged_panel: Option<String>,
    pub drag_start: Option<egui::Pos2>,
    pub current_position: Option<egui::Pos2>,
    pub target_area: Option<String>,
    pub target_position: f32,
}

impl DockingManager {
    pub fn new() -> Self {
        Self {
            dock_areas: Arc::new(RwLock::new(Vec::new())),
            panels: Arc::new(RwLock::new(std::collections::HashMap::new())),
            drag_state: Arc::new(RwLock::new(DragState::new())),
        }
    }

    pub fn create_dock_area(&self, id: String, orientation: DockOrientation, position: egui::Rect) -> DockArea {
        DockArea {
            id,
            orientation,
            position,
            panels: Vec::new(),
            splitter_positions: Vec::new(),
        }
    }

    pub fn add_dock_area(&self, dock_area: DockArea) {
        let mut dock_areas = self.dock_areas.write();
        dock_areas.push(dock_area);
    }

    pub fn dock_panel(&self, dock_area_id: &str, panel_id: &str, size_ratio: f32) -> Result<(), String> {
        let mut dock_areas = self.dock_areas.write();
        
        if let Some(dock_area) = dock_areas.iter_mut().find(|area| area.id == dock_area_id) {
            let panel = DockedPanel {
                panel_id: panel_id.to_string(),
                size_ratio,
                visible: true,
                min_size: egui::vec2(100.0, 100.0),
                max_size: None,
            };
            
            dock_area.panels.push(panel);
            Ok(())
        } else {
            Err(format!("Dock area {} not found", dock_area_id))
        }
    }

    pub fn undock_panel(&self, dock_area_id: &str, panel_id: &str) -> Result<(), String> {
        let mut dock_areas = self.dock_areas.write();
        
        if let Some(dock_area) = dock_areas.iter_mut().find(|area| area.id == dock_area_id) {
            dock_area.panels.retain(|panel| panel.panel_id != panel_id);
            Ok(())
        } else {
            Err(format!("Dock area {} not found", dock_area_id))
        }
    }

    pub fn resize_panel(&self, dock_area_id: &str, panel_id: &str, new_size_ratio: f32) -> Result<(), String> {
        let mut dock_areas = self.dock_areas.write();
        
        if let Some(dock_area) = dock_areas.iter_mut().find(|area| area.id == dock_area_id) {
            if let Some(panel) = dock_area.panels.iter_mut().find(|panel| panel.panel_id == panel_id) {
                panel.size_ratio = new_size_ratio;
                Ok(())
            } else {
                Err(format!("Panel {} not found in dock area {}", panel_id, dock_area_id))
            }
        } else {
            Err(format!("Dock area {} not found", dock_area_id))
        }
    }

    pub fn handle_drag_start(&self, panel_id: &str, pos: egui::Pos2) {
        let mut drag_state = self.drag_state.write();
        drag_state.dragging = true;
        drag_state.dragged_panel = Some(panel_id.to_string());
        drag_state.drag_start = Some(pos);
        drag_state.current_position = Some(pos);
    }

    pub fn handle_drag_move(&self, pos: egui::Pos2) {
        let mut drag_state = self.drag_state.write();
        drag_state.current_position = Some(pos);
    }

    pub fn handle_drag_end(&self) -> Option<(String, String, f32)> {
        let mut drag_state = self.drag_state.write();
        
        if let (Some(dragged_panel), Some(target_area), Some(target_position)) = 
            (drag_state.dragged_panel.clone(), drag_state.target_area.clone(), drag_state.target_position) {
            
Apply the dock operation
            let result = self.dock_panel(&target_area, &dragged_panel, target_position);
            
            drag_state.dragging = false;
            drag_state.dragged_panel = None;
            drag_state.drag_start = None;
            drag_state.current_position = None;
            drag_state.target_area = None;
            drag_state.target_position = 0.0;
            
            if result.is_ok() {
                Some((dragged_panel, target_area, target_position))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_drag_preview(&self, ui: &mut egui::Ui) {
        let drag_state = self.drag_state.read();
        
        if drag_state.dragging {
            if let (Some(dragged_panel), Some(current_pos)) = (drag_state.dragged_panel.clone(), drag_state.current_position) {
                let preview_rect = egui::Rect::from_min_size(
                    current_pos,
                    egui::vec2(150.0, 80.0)
                );
                
                let painter = ui.painter();
                painter.rect_filled(preview_rect, 4.0, egui::Color32::from_rgba_unmultiplied(100, 150, 255, 128));
                painter.rect_stroke(preview_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(50, 100, 200)));
                
                painter.text(
                    preview_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &format!("Dragging: {}", dragged_panel),
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
            }
        }
    }

    pub fn update_drop_zones(&self, ui: &mut egui::Ui) {
        let drag_state = self.drag_state.read();
        
        if drag_state.dragging {
            let dock_areas = self.dock_areas.read();
            
            for dock_area in &dock_areas {
                let drop_zone_rect = dock_area.position;
                
                if ui.rect_contains_pointer(drop_zone_rect) {
                    let painter = ui.painter();
                    painter.rect_filled(drop_zone_rect, 0.0, egui::Color32::from_rgba_unmultiplied(100, 255, 100, 50));
                    painter.rect_stroke(drop_zone_rect, 2.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(50, 200, 100)));
                    
                    let mut drag_state = self.drag_state.write();
                    drag_state.target_area = Some(dock_area.id.clone());
                    
                    if let Some(mouse_pos) = ui.pointer_hover_pos() {
                        let relative_pos = mouse_pos - drop_zone_rect.min;
                        let position_ratio = if dock_area.orientation == DockOrientation::Horizontal {
                            relative_pos.x / drop_zone_rect.width()
                        } else {
                            relative_pos.y / drop_zone_rect.height()
                        };
                        
                        drag_state.target_position = position_ratio.clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    pub fn render_dock_areas(&self, ui: &mut egui::Ui) {
        let dock_areas = self.dock_areas.read();
        
        for dock_area in &dock_areas {
            self.render_dock_area(ui, dock_area);
        }
    }

    fn render_dock_area(&self, ui: &mut egui::Ui, dock_area: &DockArea) {
        let painter = ui.painter();
        
        painter.rect_filled(dock_area.position, 4.0, egui::Color32::from_rgb(40, 40, 40));
        painter.rect_stroke(dock_area.position, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)));
        
        let mut current_x = dock_area.position.min.x;
        
        for (index, panel) in dock_area.panels.iter().enumerate() {
            let panel_width = if index < dock_area.panels.len() - 1 {
                dock_area.position.width() * panel.size_ratio
            } else {
                dock_area.position.width() - current_x
            };
            
            let panel_rect = egui::Rect::from_min_size(
                egui::pos2(current_x, dock_area.position.min.y),
                egui::vec2(panel_width, dock_area.position.height())
            );
            
            painter.rect_filled(panel_rect, 2.0, egui::Color32::from_rgb(60, 60, 60));
            painter.rect_stroke(panel_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)));
            
            painter.text(
                panel_rect.center(),
                egui::Align2::CENTER_CENTER,
                &format!("Panel: {}", panel.panel_id),
                egui::FontId::default(),
                egui::Color32::WHITE
            );
            
            current_x += panel_width;
        }
        
        for (index, &splitter_position) in dock_area.splitter_positions.iter().enumerate() {
            if index < dock_area.splitter_positions.len() {
                let splitter_x = dock_area.position.min.x + (splitter_position * dock_area.position.width());
                let splitter_rect = egui::Rect::from_min_size(
                    egui::pos2(splitter_x, dock_area.position.min.y),
                    egui::vec2(4.0, dock_area.position.height())
                );
                
                painter.rect_filled(splitter_rect, 0.0, egui::Color32::from_rgb(100, 100, 200));
            }
        }
    }

    pub fn get_dock_area(&self, id: &str) -> Option<&DockArea> {
        let dock_areas = self.dock_areas.read();
        dock_areas.iter().find(|area| area.id == id)
    }

    pub fn get_dock_area_mut(&self, id: &str) -> Option<&mut DockArea> {
        let mut dock_areas = self.dock_areas.write();
        dock_areas.iter_mut().find(|area| area.id == id)
    }

    pub fn get_panel(&self, dock_area_id: &str, panel_id: &str) -> Option<&DockedPanel> {
        if let Some(dock_area) = self.get_dock_area(dock_area_id) {
            dock_area.panels.iter().find(|panel| panel.panel_id == panel_id)
        } else {
            None
        }
    }

    pub fn get_panel_mut(&self, dock_area_id: &str, panel_id: &str) -> Option<&mut DockedPanel> {
        if let Some(dock_area) = self.get_dock_area_mut(dock_area_id) {
            dock_area.panels.iter_mut().find(|panel| panel.panel_id == panel_id)
        } else {
            None
        }
    }

    pub fn save_layout(&self) -> Result<String, String> {
        let dock_areas = self.dock_areas.read();
        let layout_data = serde_json::to_string(&*dock_areas)
            .map_err(|e| format!("Failed to serialize layout: {}", e))?;
        
        Ok(layout_data)
    }

    pub fn load_layout(&self, layout_data: &str) -> Result<(), String> {
        let dock_areas: Vec<DockArea> = serde_json::from_str(layout_data)
            .map_err(|e| format!("Failed to deserialize layout: {}", e))?;
        
        let mut current_dock_areas = self.dock_areas.write();
        *current_dock_areas = dock_areas;
        
        Ok(())
    }

    pub fn reset_layout(&self) {
        let mut dock_areas = self.dock_areas.write();
        dock_areas.clear();
        
        let mut drag_state = self.drag_state.write();
        *drag_state = DragState::new();
    }
}

impl Default for DockingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            dragging: false,
            dragged_panel: None,
            drag_start: None,
            current_position: None,
            target_area: None,
            target_position: 0.0,
        }
    }
}

impl Default for DockArea {
    fn default() -> Self {
        Self {
            id: "default_dock_area".to_string(),
            orientation: DockOrientation::Horizontal,
            position: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0)),
            panels: Vec::new(),
            splitter_positions: Vec::new(),
        }
    }
}

impl Default for DockedPanel {
    fn default() -> Self {
        Self {
            panel_id: "default_panel".to_string(),
            size_ratio: 0.5,
            visible: true,
            min_size: egui::vec2(100.0, 100.0),
            max_size: None,
        }
    }
}
