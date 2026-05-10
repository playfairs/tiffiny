use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub icon: Option<String>,
    pub data: Option<serde_json::Value>,
    pub selectable: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct TreeView {
    pub id: String,
    pub root_nodes: Vec<TreeNode>,
    pub selected_nodes: Vec<String>,
    pub expanded_nodes: std::collections::HashSet<String>,
    pub show_root: bool,
    pub multi_select: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_multi_select: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    pub on_expand: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
    pub on_context_menu: Option<Arc<dyn Fn(String, egui::Pos2) + Send + Sync>>,
}

impl TreeView {
    pub fn new(id: String) -> Self {
        Self {
            id,
            root_nodes: Vec::new(),
            selected_nodes: Vec::new(),
            expanded_nodes: std::collections::HashSet::new(),
            show_root: false,
            multi_select: false,
            enabled: true,
            visible: true,
            on_select: None,
            on_multi_select: None,
            on_expand: None,
            on_context_menu: None,
        }
    }

    pub fn add_root_node(mut self, node: TreeNode) -> Self {
        self.root_nodes.push(node);
        self
    }

    pub fn show_root(mut self, show: bool) -> Self {
        self.show_root = show;
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

    pub fn on_select(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_multi_select(mut self, callback: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.on_multi_select = Some(Arc::new(callback));
        self
    }

    pub fn on_expand(mut self, callback: impl Fn(String, bool) + Send + Sync + 'static) -> Self {
        self.on_expand = Some(Arc::new(callback));
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

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for node in &mut self.root_nodes {
                    if self.render_node(ui, node, 0) {
                        changed = true;
                    }
                }
            });

        changed
    }

    fn render_node(&mut self, ui: &mut egui::Ui, node: &mut TreeNode, depth: usize) -> bool {
        let mut changed = false;
        let indent = depth as f32 * 20.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

Expand/collapse button
            if !node.children.is_empty() {
                let expanded = self.expanded_nodes.contains(&node.id);
                let button_text = if expanded { "▼" } else { "▶" };
                
                if ui.button(button_text).clicked() {
                    if expanded {
                        self.expanded_nodes.remove(&node.id);
                    } else {
                        self.expanded_nodes.insert(node.id.clone());
                    }
                    
                    if let Some(callback) = &self.on_expand {
                        callback(node.id.clone(), !expanded);
                    }
                    
                    changed = true;
                }
            } else {
                ui.add_space(16.0);
            }

            if let Some(icon) = &node.icon {
                ui.label(icon);
            } else if node.children.is_empty() {
                ui.label("📄");
            } else {
                ui.label("📁");
            }

            let is_selected = self.selected_nodes.contains(&node.id);
            let label_color = if is_selected {
                ui.visuals().selection.bg_fill
            } else if node.enabled {
                ui.visuals().text_color()
            } else {
                ui.visuals().text_color().multiply(0.5)
            };

            let response = ui.colored_label(label_color, &node.label);
            
            if response.hovered() && ui.input(|i| i.pointer.primary_clicked()) && node.selectable && node.enabled {
                if self.multi_select {
                    if ui.input(|i| i.modifiers.shift) {
                        if is_selected {
                            self.selected_nodes.retain(|id| id != &node.id);
                        } else {
                            self.selected_nodes.push(node.id.clone());
                        }
                    } else {
                        self.selected_nodes.clear();
                        self.selected_nodes.push(node.id.clone());
                    }
                } else {
                    self.selected_nodes.clear();
                    self.selected_nodes.push(node.id.clone());
                }

                if let Some(callback) = &self.on_select {
                    callback(node.id.clone());
                }
                
                if let Some(callback) = &self.on_multi_select {
                    callback(self.selected_nodes.clone());
                }
                
                changed = true;
            }

            if response.hovered() && ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(callback) = &self.on_context_menu {
                    callback(node.id.clone(), ui.pointer_hover_pos());
                }
            }
        });

        if !node.children.is_empty() && self.expanded_nodes.contains(&node.id) {
            for child in &mut node.children {
                if self.render_node(ui, child, depth + 1) {
                    changed = true;
                }
            }
        }

        changed
    }

    pub fn get_selected_nodes(&self) -> &[String] {
        &self.selected_nodes
    }

    pub fn get_expanded_nodes(&self) -> &std::collections::HashSet<String> {
        &self.expanded_nodes
    }

    pub fn select_node(&mut self, node_id: &str) {
        if self.multi_select {
            if self.selected_nodes.contains(&node_id.to_string()) {
                self.selected_nodes.retain(|id| id != node_id);
            } else {
                self.selected_nodes.push(node_id.to_string());
            }
        } else {
            self.selected_nodes.clear();
            self.selected_nodes.push(node_id.to_string());
        }

        if let Some(callback) = &self.on_select {
            callback(node_id.to_string());
        }
        
        if let Some(callback) = &self.on_multi_select {
            callback(self.selected_nodes.clone());
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    pub fn expand_node(&mut self, node_id: &str) {
        self.expanded_nodes.insert(node_id.to_string());
        
        if let Some(callback) = &self.on_expand {
            callback(node_id.to_string(), true);
        }
    }

    pub fn collapse_node(&mut self, node_id: &str) {
        self.expanded_nodes.remove(&node_id.to_string());
        
        if let Some(callback) = &self.on_expand {
            callback(node_id.to_string(), false);
        }
    }

    pub fn expand_all(&mut self) {
        self.expand_recursive(&mut self.root_nodes);
    }

    pub fn collapse_all(&mut self) {
        self.expanded_nodes.clear();
    }

    fn expand_recursive(&mut self, nodes: &mut [TreeNode]) {
        for node in nodes.iter_mut() {
            self.expanded_nodes.insert(node.id.clone());
            self.expand_recursive(&mut node.children);
        }
    }

    pub fn find_node(&self, node_id: &str) -> Option<&TreeNode> {
        self.find_node_recursive(&self.root_nodes, node_id)
    }

    fn find_node_recursive(&self, nodes: &[TreeNode], node_id: &str) -> Option<&TreeNode> {
        for node in nodes {
            if node.id == node_id {
                return Some(node);
            }
            
            if let Some(found) = self.find_node_recursive(&node.children, node_id) {
                return Some(found);
            }
        }
        
        None
    }

    pub fn get_node_path(&self, node_id: &str) -> Vec<String> {
        self.get_node_path_recursive(&self.root_nodes, node_id, Vec::new())
    }

    fn get_node_path_recursive(&self, nodes: &[TreeNode], node_id: &str, mut path: Vec<String>) -> Vec<String> {
        for node in nodes {
            if node.id == node_id {
                path.push(node.id.clone());
                return path;
            }
            
            if let Some(mut child_path) = self.get_node_path_recursive(&node.children, node_id, path.clone()) {
                child_path.insert(0, node.id.clone());
                return child_path;
            }
        }
        
        Vec::new()
    }
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new("default_tree_view".to_string())
    }
}

impl TreeNode {
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            children: Vec::new(),
            expanded: false,
            icon: None,
            data: None,
            selectable: true,
            enabled: true,
        }
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

    pub fn add_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl Default for TreeNode {
    fn default() -> Self {
        Self::new("default_node".to_string(), "Default Node".to_string())
    }
}
