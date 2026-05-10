use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub id: String,
    pub items: Vec<ContextMenuItem>,
    pub position: egui::Pos2,
    pub visible: bool,
    pub enabled: bool,
    pub on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub separator: bool,
    pub submenu: Option<ContextMenu>,
    pub enabled: bool,
    pub checked: Option<bool>,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ContextMenu {
    pub fn new(id: String) -> Self {
        Self {
            id,
            items: Vec::new(),
            position: egui::pos2(0.0, 0.0),
            visible: false,
            enabled: true,
            on_select: None,
        }
    }

    pub fn position(mut self, position: egui::Pos2) -> Self {
        self.position = position;
        self
    }

    pub fn add_item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn add_items(mut self, items: Vec<ContextMenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn add_separator(mut self) {
        self.items.push(ContextMenuItem {
            id: format!("separator_{}", self.items.len()),
            label: String::new(),
            icon: None,
            shortcut: None,
            separator: true,
            submenu: None,
            enabled: true,
            checked: None,
            on_click: None,
        });
    }

    pub fn add_submenu(mut self, label: impl Into<String>, submenu: ContextMenu) -> Self {
        self.items.push(ContextMenuItem {
            id: format!("submenu_{}", self.items.len()),
            label: label.into(),
            icon: Some("▶".to_string()),
            shortcut: None,
            separator: false,
            submenu: Some(submenu),
            enabled: true,
            checked: None,
            on_click: None,
        });
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn show_at(&mut self, ui: &mut egui::Ui, position: egui::Pos2) -> Option<String> {
        if !self.visible {
            return None;
        }

        self.position = position;
        self.visible = true;

        let mut selected = None;

        egui::popup::show_tooltip_at_pointer(ui.ctx(), || {
            egui::Area::new(egui::Rect::from_min_size(
                self.position,
                egui::vec2(200.0, 400.0)
            ))
            .interactable(true)
            .show(ui, |ui| {
                for item in &mut self.items {
                    if let Some(item_id) = self.render_item(ui, item) {
                        selected = Some(item_id);
                    }
                }
            });
        });

        selected
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn render_item(&mut self, ui: &mut egui::Ui, item: &mut ContextMenuItem) -> Option<String> {
        if item.separator {
            ui.separator();
            return None;
        }

        let mut clicked = false;

        ui.horizontal(|ui| {
Checkbox for checkable items
            if let Some(checked) = item.checked {
                let mut check_value = *checked;
                let response = ui.checkbox(&mut check_value, "");
                if response.changed() {
                    item.checked = Some(check_value);
                }
            } else {
                if let Some(icon) = &item.icon {
                    ui.label(icon);
                    ui.add_space(8.0);
                } else {
                    ui.add_space(16.0);
                }
            }

            let label_color = if item.enabled {
                ui.visuals().text_color()
            } else {
                ui.visuals().text_color().multiply(0.5)
            };

            let response = ui.colored_label(label_color, &item.label);

            if let Some(shortcut) = &item.shortcut {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        ui.visuals().text_color().multiply(0.7),
                        shortcut
                    );
                });
            }

            if item.submenu.is_some() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("▶");
                });
            }

            if response.hovered() && ui.input(|i| i.pointer.primary_clicked()) && item.enabled {
                clicked = true;
                
                if let Some(callback) = &item.on_click {
                    callback();
                }
            }

            if response.hovered() && ui.input(|i| i.pointer.primary_clicked()) && item.submenu.is_some() {
                if let Some(submenu) = &item.submenu {
                    submenu.visible = true;
                    submenu.position = ui.pointer_hover_pos();
                }
            }
        });

        if clicked {
            if let Some(callback) = &self.on_select {
                callback(item.id.clone());
            }
            
            Some(item.id.clone())
        } else {
            None
        }
    }

    pub fn get_item(&self, item_id: &str) -> Option<&ContextMenuItem> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut ContextMenuItem> {
        self.items.iter_mut().find(|item| item.id == item_id)
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<ContextMenuItem> {
        let index = self.items.iter().position(|item| item.id == item_id);
        if let Some(index) = index {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn enable_item(&mut self, item_id: &str) -> bool {
        if let Some(item) = self.get_item_mut(item_id) {
            item.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_item(&mut self, item_id: &str) -> bool {
        if let Some(item) = self.get_item_mut(item_id) {
            item.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn set_item_checked(&mut self, item_id: &str, checked: bool) -> bool {
        if let Some(item) = self.get_item_mut(item_id) {
            item.checked = Some(checked);
            true
        } else {
            false
        }
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_visible_items(&self) -> Vec<&ContextMenuItem> {
        self.items.iter().filter(|item| item.enabled).collect()
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new("default_context_menu".to_string())
    }
}

impl ContextMenuItem {
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            icon: None,
            shortcut: None,
            separator: false,
            submenu: None,
            enabled: true,
            checked: None,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    pub fn separator() -> Self {
        Self {
            id: "separator".to_string(),
            label: String::new(),
            icon: None,
            shortcut: None,
            separator: true,
            submenu: None,
            enabled: true,
            checked: None,
            on_click: None,
        }
    }

    pub fn checkable(id: String, label: String, checked: bool, callback: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            id,
            label,
            icon: None,
            shortcut: None,
            separator: false,
            submenu: None,
            enabled: true,
            checked: Some(checked),
            on_click: Some(Arc::new(callback)),
        }
    }

    pub fn submenu(id: String, label: impl Into<String>, submenu: ContextMenu) -> Self {
        Self {
            id,
            label: label.into(),
            icon: Some("▶".to_string()),
            shortcut: None,
            separator: false,
            submenu: Some(submenu),
            enabled: true,
            checked: None,
            on_click: None,
        }
    }
}

impl Default for ContextMenuItem {
    fn default() -> Self {
        Self::new("default_item".to_string(), "Default Item".to_string())
    }
}
