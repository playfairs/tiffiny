use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ListViewItem {
    pub id: String,
    pub text: String,
    pub subtext: Option<String>,
    pub icon: Option<String>,
    pub data: Option<serde_json::Value>,
    pub selectable: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ListView {
    pub id: String,
    pub items: Vec<ListViewItem>,
    pub selected_items: Vec<String>,
    pub multi_select: bool,
    pub show_headers: bool,
    pub headers: Vec<ListViewHeader>,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_multi_select: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    pub on_double_click: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_context_menu: Option<Arc<dyn Fn(String, egui::Pos2) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ListViewHeader {
    pub id: String,
    pub text: String,
    pub width: f32,
    pub sortable: bool,
    pub alignment: HeaderAlignment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderAlignment {
    Left,
    Center,
    Right,
}

impl ListView {
    pub fn new(id: String) -> Self {
        Self {
            id,
            items: Vec::new(),
            selected_items: Vec::new(),
            multi_select: false,
            show_headers: false,
            headers: Vec::new(),
            sort_column: None,
            sort_ascending: true,
            enabled: true,
            visible: true,
            on_select: None,
            on_multi_select: None,
            on_double_click: None,
            on_context_menu: None,
        }
    }

    pub fn add_item(mut self, item: ListViewItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn add_items(mut self, items: Vec<ListViewItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn clear_items(mut self) -> Self {
        self.items.clear();
        self.selected_items.clear();
        self
    }

    pub fn set_headers(mut self, headers: Vec<ListViewHeader>) -> Self {
        self.headers = headers;
        self.show_headers = !headers.is_empty();
        self
    }

    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi_select = multi;
        if !multi {
            self.selected_items.clear();
        }
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

    pub fn on_select(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_multi_select(mut self, callback: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.on_multi_select = Some(Arc::new(callback));
        self
    }

    pub fn on_double_click(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_double_click = Some(Arc::new(callback));
        self
    }

    pub fn on_context_menu(mut self, callback: impl Fn(String, egui::Pos2) + Send + Sync + 'static) -> Self {
        self.on_context_menu = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

Sort items if needed
        if let Some(sort_column) = self.sort_column {
            self.sort_items(sort_column);
        }

        if self.show_headers {
            self.render_headers(ui);
            ui.separator();
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, item) in self.items.iter().enumerate() {
                    if self.render_item(ui, item, index) {
                        changed = true;
                    }
                }
            });

        changed
    }

    fn render_headers(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (index, header) in self.headers.iter().enumerate() {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(header.width, 24.0),
                    egui::Sense::click()
                );

                let painter = ui.painter();
                painter.rect_filled(rect, 0.0, ui.visuals().header_bg_color);
                painter.rect_stroke(rect, 0.0, ui.visuals().header_stroke);

                let text_pos = match header.alignment {
                    HeaderAlignment::Left => egui::pos2(rect.min.x + 8.0, rect.center().y),
                    HeaderAlignment::Center => egui::pos2(rect.center().x, rect.center().y),
                    HeaderAlignment::Right => egui::pos2(rect.max.x - 8.0, rect.center().y),
                };

                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    &header.text,
                    egui::FontId::default(),
                    ui.visuals().text_color()
                );

                if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) && header.sortable {
                    if let Some(current_sort) = self.sort_column {
                        if current_sort == index {
                            self.sort_ascending = !self.sort_ascending;
                        } else {
                            self.sort_column = Some(index);
                            self.sort_ascending = true;
                        }
                    } else {
                        self.sort_column = Some(index);
                        self.sort_ascending = true;
                    }
                }
            }
        });
    }

    fn render_item(&mut self, ui: &mut egui::Ui, item: &ListViewItem, index: usize) -> bool {
        let mut changed = false;
        let is_selected = self.selected_items.contains(&item.id);

        ui.horizontal(|ui| {
            if is_selected {
                ui.colored_label(ui.visuals().selection.bg_fill, "▶");
            } else {
                ui.label("  ");
            }

            if let Some(icon) = &item.icon {
                ui.label(icon);
            } else {
                ui.label("📄");
            }

            ui.vertical(|ui| {
                let text_color = if item.enabled {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().text_color().multiply(0.5)
                };

                ui.colored_label(text_color, &item.text);

                if let Some(subtext) = &item.subtext {
                    ui.colored_label(
                        ui.visuals().text_color().multiply(0.7),
                        subtext
                    );
                }
            });

            let response = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 24.0),
                egui::Sense::click()
            );

            if ui.rect_contains_pointer(response.rect) {
                if ui.input(|i| i.pointer.primary_clicked()) && item.selectable && item.enabled {
                    if self.multi_select {
                        if ui.input(|i| i.modifiers.shift) {
                            if is_selected {
                                self.selected_items.retain(|id| id != &item.id);
                            } else {
                                self.selected_items.push(item.id.clone());
                            }
                        } else {
                            self.selected_items.clear();
                            self.selected_items.push(item.id.clone());
                        }
                    } else {
                        self.selected_items.clear();
                        self.selected_items.push(item.id.clone());
                    }

                    changed = true;

                    if let Some(callback) = &self.on_multi_select {
                        callback(self.selected_items.clone());
                    }
                } else if !self.multi_select {
                    self.selected_items.clear();
                    self.selected_items.push(item.id.clone());

                    changed = true;

                    if let Some(callback) = &self.on_select {
                        callback(item.id.clone());
                    }
                }
            }

            if ui.rect_contains_pointer(response.rect) && ui.input(|i| i.pointer.double_clicked()) {
                if let Some(callback) = &self.on_double_click {
                    callback(item.id.clone());
                }
            }

            if ui.rect_contains_pointer(response.rect) && ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(callback) = &self.on_context_menu {
                    callback(item.id.clone(), ui.pointer_hover_pos());
                }
            }
        });

        changed
    }

    fn sort_items(&mut self, column: usize) {
        if column >= self.headers.len() {
            return;
        }

        self.items.sort_by(|a, b| {
            let a_text = if column == 0 { &a.text } else { &a.subtext.as_deref().unwrap_or("") };
            let b_text = if column == 0 { &b.text } else { &b.subtext.as_deref().unwrap_or("") };

            if self.sort_ascending {
                a_text.cmp(b_text)
            } else {
                b_text.cmp(a_text)
            }
        });
    }

    pub fn get_selected_items(&self) -> &[String] {
        &self.selected_items
    }

    pub fn get_selected_item(&self) -> Option<&String> {
        if self.selected_items.len() == 1 {
            self.selected_items.first()
        } else {
            None
        }
    }

    pub fn select_item(&mut self, item_id: &str) {
        if self.multi_select {
            if !self.selected_items.contains(&item_id.to_string()) {
                self.selected_items.push(item_id.to_string());
            }
        } else {
            self.selected_items.clear();
            self.selected_items.push(item_id.to_string());
        }
    }

    pub fn select_all(&mut self) {
        self.selected_items.clear();
        for item in &self.items {
            if item.selectable && item.enabled {
                self.selected_items.push(item.id.clone());
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    pub fn get_item(&self, item_id: &str) -> Option<&ListViewItem> {
        self.items.iter().find(|item| item.id == item_id)
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<ListViewItem> {
        let index = self.items.iter().position(|item| item.id == item_id);
        if let Some(index) = index {
            let item = self.items.remove(index);
            self.selected_items.retain(|id| id != item_id);
            item
        } else {
            None
        }
    }

    pub fn update_item(&mut self, item_id: &str, updater: impl Fn(&mut ListViewItem)) -> bool {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == item_id) {
            updater(item);
            true
        } else {
            false
        }
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new("default_list_view".to_string())
    }
}

impl ListViewItem {
    pub fn new(id: String, text: String) -> Self {
        Self {
            id,
            text,
            subtext: None,
            icon: None,
            data: None,
            selectable: true,
            enabled: true,
        }
    }

    pub fn subtext(mut self, subtext: impl Into<String>) -> Self {
        self.subtext = Some(subtext.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for ListViewItem {
    fn default() -> Self {
        Self::new("default_item".to_string(), "Default Item".to_string())
    }
}

impl ListViewHeader {
    pub fn new(id: String, text: String, width: f32) -> Self {
        Self {
            id,
            text,
            width,
            sortable: true,
            alignment: HeaderAlignment::Left,
        }
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn alignment(mut self, alignment: HeaderAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Default for ListViewHeader {
    fn default() -> Self {
        Self::new("default_header".to_string(), "Header".to_string(), 100.0)
    }
}
