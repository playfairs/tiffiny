pub mod button;
pub mod slider;
pub mod text_input;
pub mod color_picker;
pub mod file_browser;
pub mod progress_bar;
pub mod tree_view;
pub mod list_view;
pub mod menu;
pub mod toolbar;
pub mod status_bar;
pub mod tooltip;
pub mod modal;
pub mod context_menu;

use std::sync::Arc;
use parking_lot::RwLock;

pub struct WidgetManager {
    pub widgets: Arc<RwLock<Vec<Widget>>>,
    pub focused_widget: Arc<RwLock<Option<String>>>,
    pub theme: Arc<RwLock<crate::theme::Theme>>,
}

#[derive(Debug, Clone)]
pub struct Widget {
    pub id: String,
    pub widget_type: WidgetType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetType {
    Button,
    Slider,
    TextInput,
    ColorPicker,
    FileBrowser,
    ProgressBar,
    TreeView,
    ListView,
    Menu,
    Toolbar,
    StatusBar,
    Tooltip,
    Modal,
    ContextMenu,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            widgets: Arc::new(RwLock::new(Vec::new())),
            focused_widget: Arc::new(RwLock::new(None)),
            theme: Arc::new(RwLock::new(crate::theme::Theme::default())),
        }
    }

    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn add_widget(&self, widget: Widget) {
        let mut widgets = self.widgets.write();
        widgets.push(widget);
    }

    pub fn remove_widget(&self, widget_id: &str) -> Option<Widget> {
        let mut widgets = self.widgets.write();
        let index = widgets.iter().position(|w| w.id == widget_id);
        if let Some(index) = index {
            Some(widgets.remove(index))
        } else {
            None
        }
    }

    pub fn get_widget(&self, widget_id: &str) -> Option<Widget> {
        let widgets = self.widgets.read();
        widgets.iter().find(|w| w.id == widget_id).cloned()
    }

    pub fn set_focused_widget(&self, widget_id: Option<String>) {
        let mut focused = self.focused_widget.write();
        *focused = widget_id;
    }

    pub fn get_focused_widget(&self) -> Option<String> {
        let focused = self.focused_widget.read();
        focused.clone()
    }

    pub fn set_theme(&self, theme: crate::theme::Theme) {
        let mut theme_guard = self.theme.write();
        *theme_guard = theme;
    }

    pub fn get_theme(&self) -> crate::theme::Theme {
        let theme_guard = self.theme.read();
        theme_guard.clone()
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}
