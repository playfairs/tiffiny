use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct Menu {
    pub id: String,
    pub items: Vec<MenuItem>,
    pub style: MenuStyle,
    pub visible: bool,
    pub enabled: bool,
    pub on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
    pub enabled: bool,
    pub separator: bool,
    pub submenu: Option<Menu>,
    pub checked: Option<bool>,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuStyle {
    Horizontal,
    Vertical,
    Context,
    Menubar,
}

impl Menu {
    pub fn new(id: String) -> Self {
        Self {
            id,
            items: Vec::new(),
            style: MenuStyle::Vertical,
            visible: true,
            enabled: true,
            on_select: None,
        }
    }

    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }

    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn add_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
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

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<String> {
        if !self.visible {
            return None;
        }

        let mut selected = None;

        match self.style {
            MenuStyle::Horizontal => {
                ui.horizontal(|ui| {
                    for item in &mut self.items {
                        if let Some(item_id) = self.render_menu_item(ui, item) {
                            selected = Some(item_id);
                        }
                    }
                });
            },
            MenuStyle::Vertical => {
                for item in &mut self.items {
                    if let Some(item_id) = self.render_menu_item(ui, item) {
                        selected = Some(item_id);
                    }
                }
            },
            MenuStyle::Context => {
                egui::popup::show_tooltip_at_pointer(ui.ctx(), |ui| {
                    for item in &mut self.items {
                        if let Some(item_id) = self.render_menu_item(ui, item) {
                            selected = Some(item_id);
                        }
                    }
                });
            },
            MenuStyle::Menubar => {
                ui.horizontal(|ui| {
                    for item in &mut self.items {
                        if let Some(item_id) = self.render_menubar_item(ui, item) {
                            selected = Some(item_id);
                        }
                    }
                });
            },
        }

        selected
    }

    fn render_menu_item(&mut self, ui: &mut egui::Ui, item: &mut MenuItem) -> Option<String> {
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
                    if let Some(index) = self.items.iter().position(|i| i.id == item.id) {
                        if let Some(checkable_item) = self.items.get_mut(index) {
                            checkable_item.checked = Some(check_value);
                        }
                    }
                }
            } else {
                if let Some(icon) = &item.icon {
                    ui.label(icon);
                    ui.add_space(8.0);
                }

                let label_color = if item.enabled {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().text_color().multiply(0.5)
                };

                ui.colored_label(label_color, &item.label);

                if let Some(shortcut) = &item.shortcut {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(
                            ui.visuals().text_color().multiply(0.7),
                            shortcut
                        );
                    });
                }
            }

            let response = ui.allocate_response(
                egui::vec2(ui.available_width(), 24.0),
                egui::Sense::click()
            );

            if response.hovered() {
                ui.painter().rect_filled(
                    response.rect,
                    0.0,
                    ui.visuals().hover_bg_color()
                );
            }

            if response.clicked() && item.enabled {
                clicked = true;
                
                if let Some(callback) = &item.on_click {
                    callback();
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

    fn render_menubar_item(&mut self, ui: &mut egui::Ui, item: &mut MenuItem) -> Option<String> {
        if item.separator {
            ui.separator();
            return None;
        }

        let mut clicked = false;

        let response = if let Some(submenu) = &item.submenu {
            ui.menu_button(&item.label, |ui| {
                if let Some(item_id) = submenu.render(ui) {
                }
            })
        } else {
            let mut button_text = &item.label;
            
            if let Some(icon) = &item.icon {
                button_text = &format!("{} {}", icon, item.label);
            }

            if let Some(shortcut) = &item.shortcut {
                button_text = &format!("{} ({})", button_text, shortcut);
            }

            let response = ui.add_enabled(item.enabled, egui::Button::new(button_text));
            
            if response.hovered() {
                if let Some(callback) = &item.on_click {
                    ui.label(format!("Click to execute: {}", item.label));
                }
            }

            response
        };

        if response.clicked() && item.enabled {
            clicked = true;
            
            if let Some(callback) = &item.on_click {
                callback();
            }
        }

        if clicked {
            if let Some(callback) = &self.on_select {
                callback(item.id.clone());
            }
            
            Some(item.id.clone())
        } else {
            None
        }
    }

    pub fn add_separator(&mut self) {
        self.items.push(MenuItem {
            id: format!("separator_{}", self.items.len()),
            label: String::new(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: true,
            submenu: None,
            checked: None,
            on_click: None,
        });
    }

    pub fn add_submenu(&mut self, label: impl Into<String>, submenu: Menu) {
        self.items.push(MenuItem {
            id: format!("submenu_{}", self.items.len()),
            label: label.into(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: false,
            submenu: Some(submenu),
            checked: None,
            on_click: None,
        });
    }

    pub fn add_checkable_item(&mut self, id: String, label: impl Into<String>, checked: bool, callback: impl Fn() + Send + Sync + 'static) {
        self.items.push(MenuItem {
            id,
            label: label.into(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: false,
            submenu: None,
            checked: Some(checked),
            on_click: Some(Arc::new(callback)),
        });
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn get_item(&self, item_id: &str) -> Option<&MenuItem> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut MenuItem> {
        self.items.iter_mut().find(|item| item.id == item_id)
    }

    pub fn remove_item(&mut self, item_id: &str) -> bool {
        let index = self.items.iter().position(|item| item.id == item_id);
        if let Some(index) = index {
            self.items.remove(index);
            true
        } else {
            false
        }
    }

    pub fn enable_item(&mut self, item_id: &str, enabled: bool) -> bool {
        if let Some(item) = self.get_item_mut(item_id) {
            item.enabled = enabled;
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
}

impl Default for Menu {
    fn default() -> Self {
        Self::new("default_menu".to_string())
    }
}

impl MenuItem {
    pub fn new(id: String, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: false,
            submenu: None,
            checked: None,
            on_click: None,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
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
            shortcut: None,
            icon: None,
            enabled: true,
            separator: true,
            submenu: None,
            checked: None,
            on_click: None,
        }
    }

    pub fn submenu(id: String, label: impl Into<String>, submenu: Menu) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: false,
            submenu: Some(submenu),
            checked: None,
            on_click: None,
        }
    }

    pub fn checkable(id: String, label: impl Into<String>, checked: bool, callback: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            icon: None,
            enabled: true,
            separator: false,
            submenu: None,
            checked: Some(checked),
            on_click: Some(Arc::new(callback)),
        }
    }
}

impl Default for MenuItem {
    fn default() -> Self {
        Self::new("default_item".to_string(), "Default Item")
    }
}

pub struct MenuBar {
    pub menus: Vec<Menu>,
    pub visible: bool,
    pub enabled: bool,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            visible: true,
            enabled: true,
        }
    }

    pub fn add_menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if !self.visible {
            return;
        }

        egui::menu::bar(ui, |ui| {
            for menu in &mut self.menus {
                menu.render(ui);
            }
        });
    }

    pub fn get_menu(&self, menu_id: &str) -> Option<&Menu> {
        self.menus.iter().find(|menu| menu.id == menu_id)
    }

    pub fn get_menu_mut(&mut self, menu_id: &str) -> Option<&mut Menu> {
        self.menus.iter_mut().find(|menu| menu.id == menu_id)
    }

    pub fn clear(&mut self) {
        self.menus.clear();
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}
