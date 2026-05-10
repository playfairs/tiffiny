use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ToolBar {
    pub id: String,
    pub items: Vec<ToolBarItem>,
    pub orientation: ToolBarOrientation,
    pub show_labels: bool,
    pub show_icons: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_action: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ToolBarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub shortcut: Option<String>,
    pub separator: bool,
    pub group: Option<String>,
    pub enabled: bool,
    pub visible: bool,
    pub checkable: bool,
    pub checked: bool,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolBarOrientation {
    Horizontal,
    Vertical,
}

impl ToolBar {
    pub fn new(id: String) -> Self {
        Self {
            id,
            items: Vec::new(),
            orientation: ToolBarOrientation::Horizontal,
            show_labels: true,
            show_icons: true,
            enabled: true,
            visible: true,
            on_action: None,
        }
    }

    pub fn add_item(mut self, item: ToolBarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn add_items(mut self, items: Vec<ToolBarItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn orientation(mut self, orientation: ToolBarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
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

    pub fn on_action(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_action = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<String> {
        if !self.visible {
            return None;
        }

        let mut action = None;

        match self.orientation {
            ToolBarOrientation::Horizontal => {
                ui.horizontal(|ui| {
                    action = self.render_items_horizontal(ui);
                });
            },
            ToolBarOrientation::Vertical => {
                ui.vertical(|ui| {
                    action = self.render_items_vertical(ui);
                });
            },
        }

        action
    }

    fn render_items_horizontal(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut current_group: Option<String> = None;
        let mut action = None;

        for item in &mut self.items {
            if !item.visible {
                continue;
            }

Handle group separators
            if let Some(group) = &item.group {
                if current_group.as_ref() != Some(group) {
                    ui.separator();
                    if !group.is_empty() {
                        ui.label(group);
                        ui.add_space(8.0);
                    }
                    current_group = Some(group.clone());
                }
            } else {
                current_group = None;
            }

            if item.separator {
                ui.add_space(8.0);
                continue;
            }

            let item_action = self.render_item_horizontal(ui, item);
            if let Some(item_action) = item_action {
                action = item_action;
            }
        }

        action
    }

    fn render_items_vertical(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut current_group: Option<String> = None;
        let mut action = None;

        for item in &mut self.items {
            if !item.visible {
                continue;
            }

            if let Some(group) = &item.group {
                if current_group.as_ref() != Some(group) {
                    ui.separator();
                    if !group.is_empty() {
                        ui.label(group);
                    }
                    current_group = Some(group.clone());
                }
            } else {
                current_group = None;
            }

            if item.separator {
                ui.separator();
                continue;
            }

            let item_action = self.render_item_vertical(ui, item);
            if let Some(item_action) = item_action {
                action = item_action;
            }
        }

        action
    }

    fn render_item_horizontal(&mut self, ui: &mut egui::Ui, item: &mut ToolBarItem) -> Option<String> {
        let mut action = None;

        ui.horizontal(|ui| {
            if self.show_icons && item.icon.is_some() {
                let icon_response = ui.add_enabled(
                    item.enabled,
                    egui::Button::new(item.icon.as_ref().unwrap_or(""))
                );

                if icon_response.hovered() {
                    if let Some(tooltip) = &item.tooltip {
                        icon_response = icon_response.on_hover_text(tooltip);
                    }
                }

                if icon_response.clicked() && item.enabled {
                    if let Some(callback) = &item.on_click {
                        callback();
                    }
                    action = Some(item.id.clone());
                }
            }

            if self.show_labels && !item.label.is_empty() {
                let label_response = ui.add_enabled(
                    item.enabled,
                    egui::Button::new(&item.label)
                );

                if label_response.hovered() {
                    if let Some(tooltip) = &item.tooltip {
                        label_response = label_response.on_hover_text(tooltip);
                    }
                }

                if label_response.clicked() && item.enabled {
                    if let Some(callback) = &item.on_click {
                        callback();
                    }
                    action = Some(item.id.clone());
                }
            }

            if item.checkable {
                let mut checked = item.checked;
                let checkbox_response = ui.add_enabled(
                    item.enabled,
                    egui::Checkbox::new(&mut checked, "")
                );

                if checkbox_response.changed() {
                    item.checked = checked;
                    if let Some(callback) = &item.on_toggle {
                        callback(checked);
                    }
                    action = Some(item.id.clone());
                }
            }

            if let Some(shortcut) = &item.shortcut {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        ui.visuals().text_color().multiply(0.7),
                        shortcut
                    );
                });
            }
        });

        action
    }

    fn render_item_vertical(&mut self, ui: &mut egui::Ui, item: &mut ToolBarItem) -> Option<String> {
        let mut action = None;

        if item.separator {
            ui.separator();
            return None;
        }

        let button_content = if self.show_icons && item.icon.is_some() {
            if self.show_labels && !item.label.is_empty() {
                format!("{} {}", item.icon.as_ref().unwrap_or(""), item.label)
            } else {
                item.icon.as_ref().unwrap_or("").to_string()
            }
        } else {
            item.label.clone()
        };

        let button_response = ui.add_enabled(
            item.enabled,
            egui::Button::new(button_content)
        );

        if button_response.hovered() {
            if let Some(tooltip) = &item.tooltip {
                button_response = button_response.on_hover_text(tooltip);
            }
        }

        if button_response.clicked() && item.enabled {
            if let Some(callback) = &item.on_click {
                callback();
            }
            action = Some(item.id.clone());
        }

        if let Some(shortcut) = &item.shortcut {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.colored_label(
                    ui.visuals().text_color().multiply(0.7),
                    shortcut
                );
            });
        }

        action
    }

    pub fn get_item(&self, item_id: &str) -> Option<&ToolBarItem> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut ToolBarItem> {
        self.items.iter_mut().find(|item| item.id == item_id)
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<ToolBarItem> {
        let index = self.items.iter().position(|item| item.id == item_id);
        if let Some(index) = index {
            Some(self.items.remove(index))
        } else {
            None
        }
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
            item.checked = checked;
            true
        } else {
            false
        }
    }

    pub fn add_separator(&mut self) {
        self.items.push(ToolBarItem {
            id: format!("separator_{}", self.items.len()),
            label: String::new(),
            icon: None,
            tooltip: None,
            shortcut: None,
            separator: true,
            group: None,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            on_click: None,
            on_toggle: None,
        });
    }

    pub fn add_group(&mut self, group_name: &str) {
        self.items.push(ToolBarItem {
            id: format!("group_{}", self.items.len()),
            label: String::new(),
            icon: None,
            tooltip: None,
            shortcut: None,
            separator: false,
            group: Some(group_name.to_string()),
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            on_click: None,
            on_toggle: None,
        });
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn get_items(&self) -> &[ToolBarItem] {
        &self.items
    }

    pub fn get_visible_items(&self) -> Vec<&ToolBarItem> {
        self.items.iter().filter(|item| item.visible).collect()
    }
}

impl Default for ToolBar {
    fn default() -> Self {
        Self::new("default_toolbar".to_string())
    }
}

impl ToolBarItem {
    pub fn new(id: String, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            tooltip: None,
            shortcut: None,
            separator: false,
            group: None,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            on_click: None,
            on_toggle: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
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

    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    pub fn on_toggle(mut self, callback: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(callback));
        self
    }

    pub fn separator() -> Self {
        Self {
            id: "separator".to_string(),
            label: String::new(),
            icon: None,
            tooltip: None,
            shortcut: None,
            separator: true,
            group: None,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            on_click: None,
            on_toggle: None,
        }
    }

    pub fn group(name: impl Into<String>) -> Self {
        Self {
            id: format!("group_{}", name.into()),
            label: String::new(),
            icon: None,
            tooltip: None,
            shortcut: None,
            separator: false,
            group: Some(name.into()),
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            on_click: None,
            on_toggle: None,
        }
    }
}

impl Default for ToolBarItem {
    fn default() -> Self {
        Self::new("default_item".to_string(), "Default Item")
    }
}
