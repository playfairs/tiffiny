use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ShortcutManager {
    pub shortcuts: Arc<RwLock<std::collections::HashMap<String, Shortcut>>>,
    pub active_shortcuts: Arc<RwLock<std::collections::HashSet<String>>>,
    pub enabled: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct Shortcut {
    pub id: String,
    pub key_combination: KeyCombination,
    pub action: ShortcutAction,
    pub description: String,
    pub category: ShortcutCategory,
    pub enabled: bool,
    pub global: bool,
}

#[derive(Debug, Clone)]
pub enum KeyCombination {
    Single(egui::Key),
    Modifier(ModifierKey, egui::Key),
    Multiple(Vec<egui::Key>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Cmd,
}

#[derive(Debug, Clone)]
pub enum ShortcutAction {
    Command(String),
    Custom(Arc<dyn Fn() + Send + Sync>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutCategory {
    File,
    Edit,
    View,
    Navigation,
    Playback,
    Timeline,
    Effects,
    Tools,
    Window,
    Custom,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut shortcuts = std::collections::HashMap::new();
        
Add default shortcuts
        shortcuts.insert("new_project".to_string(), Shortcut {
            id: "new_project".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::N),
            action: ShortcutAction::Command("new_project".to_string()),
            description: "Create new project".to_string(),
            category: ShortcutCategory::File,
            enabled: true,
            global: false,
        });

        shortcuts.insert("open_project".to_string(), Shortcut {
            id: "open_project".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::O),
            action: ShortcutAction::Command("open_project".to_string()),
            description: "Open existing project".to_string(),
            category: ShortcutCategory::File,
            enabled: true,
            global: false,
        });

        shortcuts.insert("save_project".to_string(), Shortcut {
            id: "save_project".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::S),
            action: ShortcutAction::Command("save_project".to_string()),
            description: "Save current project".to_string(),
            category: ShortcutCategory::File,
            enabled: true,
            global: false,
        });

        shortcuts.insert("undo".to_string(), Shortcut {
            id: "undo".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::Z),
            action: ShortcutAction::Command("undo".to_string()),
            description: "Undo last action".to_string(),
            category: ShortcutCategory::Edit,
            enabled: true,
            global: false,
        });

        shortcuts.insert("redo".to_string(), Shortcut {
            id: "redo".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::Y),
            action: ShortcutAction::Command("redo".to_string()),
            description: "Redo last action".to_string(),
            category: ShortcutCategory::Edit,
            enabled: true,
            global: false,
        });

        shortcuts.insert("play_pause".to_string(), Shortcut {
            id: "play_pause".to_string(),
            key_combination: KeyCombination::Single(egui::Key::Space),
            action: ShortcutAction::Command("play_pause".to_string()),
            description: "Play or pause playback".to_string(),
            category: ShortcutCategory::Playback,
            enabled: true,
            global: false,
        });

        shortcuts.insert("stop".to_string(), Shortcut {
            id: "stop".to_string(),
            key_combination: KeyCombination::Single(egui::Key::K),
            action: ShortcutAction::Command("stop".to_string()),
            description: "Stop playback".to_string(),
            category: ShortcutCategory::Playback,
            enabled: true,
            global: false,
        });

        shortcuts.insert("previous_frame".to_string(), Shortcut {
            id: "previous_frame".to_string(),
            key_combination: KeyCombination::Single(egui::Key::J),
            action: ShortcutAction::Command("previous_frame".to_string()),
            description: "Go to previous frame".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
            global: false,
        });

        shortcuts.insert("next_frame".to_string(), Shortcut {
            id: "next_frame".to_string(),
            key_combination: KeyCombination::Single(egui::Key::L),
            action: ShortcutAction::Command("next_frame".to_string()),
            description: "Go to next frame".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
            global: false,
        });

        shortcuts.insert("zoom_in".to_string(), Shortcut {
            id: "zoom_in".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::Plus),
            action: ShortcutAction::Command("zoom_in".to_string()),
            description: "Zoom in".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
            global: false,
        });

        shortcuts.insert("zoom_out".to_string(), Shortcut {
            id: "zoom_out".to_string(),
            key_combination: KeyCombination::Modifier(ModifierKey::Ctrl, egui::Key::Minus),
            action: ShortcutAction::Command("zoom_out".to_string()),
            description: "Zoom out".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
            global: false,
        });

        shortcuts.insert("delete".to_string(), Shortcut {
            id: "delete".to_string(),
            key_combination: KeyCombination::Single(egui::Key::Delete),
            action: ShortcutAction::Command("delete".to_string()),
            description: "Delete selected item".to_string(),
            category: ShortcutCategory::Edit,
            enabled: true,
            global: false,
        });

        Self {
            shortcuts: Arc::new(RwLock::new(shortcuts)),
            active_shortcuts: Arc::new(RwLock::new(std::collections::HashSet::new())),
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub fn add_shortcut(&self, shortcut: Shortcut) -> Result<(), String> {
        let mut shortcuts = self.shortcuts.write();
        
        if shortcuts.contains_key(&shortcut.id) {
            return Err(format!("Shortcut with ID {} already exists", shortcut.id));
        }
        
        shortcuts.insert(shortcut.id.clone(), shortcut);
        Ok(())
    }

    pub fn remove_shortcut(&self, shortcut_id: &str) -> Option<Shortcut> {
        let mut shortcuts = self.shortcuts.write();
        shortcuts.remove(shortcut_id)
    }

    pub fn get_shortcut(&self, shortcut_id: &str) -> Option<Shortcut> {
        let shortcuts = self.shortcuts.read();
        shortcuts.get(shortcut_id).cloned()
    }

    pub fn update_shortcut(&self, shortcut: Shortcut) -> Result<(), String> {
        let mut shortcuts = self.shortcuts.write();
        
        if !shortcuts.contains_key(&shortcut.id) {
            return Err(format!("Shortcut with ID {} not found", shortcut.id));
        }
        
        shortcuts.insert(shortcut.id.clone(), shortcut);
        Ok(())
    }

    pub fn enable_shortcut(&self, shortcut_id: &str) -> bool {
        let mut shortcuts = self.shortcuts.write();
        
        if let Some(shortcut) = shortcuts.get_mut(shortcut_id) {
            shortcut.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_shortcut(&self, shortcut_id: &str) -> bool {
        let mut shortcuts = self.shortcuts.write();
        
        if let Some(shortcut) = shortcuts.get_mut(shortcut_id) {
            shortcut.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled = self.enabled.write();
        *enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    pub fn check_key_press(&self, key: &egui::Key, modifiers: &egui::Modifiers) -> Vec<String> {
        if !self.is_enabled() {
            return Vec::new();
        }

        let shortcuts = self.shortcuts.read();
        let mut triggered_shortcuts = Vec::new();

        for (shortcut_id, shortcut) in shortcuts.iter() {
            if !shortcut.enabled {
                continue;
            }

            if self.matches_key_combination(&shortcut.key_combination, key, modifiers) {
                triggered_shortcuts.push(shortcut_id.clone());
            }
        }

        triggered_shortcuts
    }

    fn matches_key_combination(&self, combination: &KeyCombination, key: &egui::Key, modifiers: &egui::Modifiers) -> bool {
        match combination {
            KeyCombination::Single(combination_key) => key == combination_key,
            KeyCombination::Modifier(modifier_key, combination_key) => {
                let has_modifier = match modifier_key {
                    ModifierKey::Ctrl => modifiers.ctrl,
                    ModifierKey::Alt => modifiers.alt,
                    ModifierKey::Shift => modifiers.shift,
                    ModifierKey::Cmd => modifiers.command,
                };
                has_modifier && key == combination_key
            },
            KeyCombination::Multiple(keys) => keys.contains(key),
        }
    }

    pub fn execute_shortcut(&self, shortcut_id: &str) -> bool {
        let shortcuts = self.shortcuts.read();
        
        if let Some(shortcut) = shortcuts.get(shortcut_id) {
            if !shortcut.enabled {
                return false;
            }

            match &shortcut.action {
                ShortcutAction::Command(command) => {
                    tracing::info!("Executing command: {}", command);
                    true
                },
                ShortcutAction::Custom(callback) => {
                    callback();
                    true
                },
            }
        } else {
            false
        }
    }

    pub fn get_shortcuts_by_category(&self, category: ShortcutCategory) -> Vec<Shortcut> {
        let shortcuts = self.shortcuts.read();
        shortcuts
            .values()
            .filter(|shortcut| shortcut.category == category)
            .cloned()
            .collect()
    }

    pub fn get_all_shortcuts(&self) -> Vec<Shortcut> {
        let shortcuts = self.shortcuts.read();
        shortcuts.values().cloned().collect()
    }

    pub fn export_shortcuts(&self) -> Result<String, String> {
        let shortcuts = self.shortcuts.read();
        serde_json::to_string(&*shortcuts)
            .map_err(|e| format!("Failed to export shortcuts: {}", e))
    }

    pub fn import_shortcuts(&self, data: &str) -> Result<(), String> {
        let imported_shortcuts: std::collections::HashMap<String, Shortcut> = serde_json::from_str(data)
            .map_err(|e| format!("Failed to import shortcuts: {}", e))?;
        
        let mut shortcuts = self.shortcuts.write();
        for (id, shortcut) in imported_shortcuts {
            shortcuts.insert(id, shortcut);
        }
        
        Ok(())
    }

    pub fn reset_to_defaults(&self) {
        let mut shortcuts = self.shortcuts.write();
        shortcuts.clear();

        let default_manager = Self::new();
        let default_shortcuts = default_manager.shortcuts.read();
        
        for (id, shortcut) in default_shortcuts.iter() {
            shortcuts.insert(id.clone(), shortcut.clone());
        }
    }

    pub fn validate_shortcuts(&self) -> Vec<String> {
        let shortcuts = self.shortcuts.read();
        let mut conflicts = Vec::new();
        let mut key_combinations = std::collections::HashMap::new();

        for (shortcut_id, shortcut) in shortcuts.iter() {
            if !shortcut.enabled {
                continue;
            }

            let key_str = format!("{:?}", shortcut.key_combination);
            
            if let Some(existing_id) = key_combinations.get(&key_str) {
                conflicts.push(format!(
                    "Shortcut '{}' conflicts with '{}'",
                    shortcut_id, existing_id
                ));
            } else {
                key_combinations.insert(key_str, shortcut_id.clone());
            }
        }

        conflicts
    }

    pub fn get_shortcut_help(&self, shortcut_id: &str) -> Option<String> {
        let shortcuts = self.shortcuts.read();
        
        if let Some(shortcut) = shortcuts.get(shortcut_id) {
            Some(format!(
                "{}: {} ({})",
                shortcut.description,
                format_key_combination(&shortcut.key_combination),
                format!("{:?}", shortcut.category)
            ))
        } else {
            None
        }
    }

    pub fn get_category_name(&self, category: ShortcutCategory) -> &'static str {
        match category {
            ShortcutCategory::File => "File",
            ShortcutCategory::Edit => "Edit",
            ShortcutCategory::View => "View",
            ShortcutCategory::Navigation => "Navigation",
            ShortcutCategory::Playback => "Playback",
            ShortcutCategory::Timeline => "Timeline",
            ShortcutCategory::Effects => "Effects",
            ShortcutCategory::Tools => "Tools",
            ShortcutCategory::Window => "Window",
            ShortcutCategory::Custom => "Custom",
        }
    }
}

fn format_key_combination(combination: &KeyCombination) -> String {
    match combination {
        KeyCombination::Single(key) => format!("{:?}", key),
        KeyCombination::Modifier(modifier, key) => {
            let modifier_str = match modifier {
                ModifierKey::Ctrl => "Ctrl",
                ModifierKey::Alt => "Alt",
                ModifierKey::Shift => "Shift",
                ModifierKey::Cmd => "Cmd",
            };
            format!("{}+{:?}", modifier_str, key)
        },
        KeyCombination::Multiple(keys) => {
            keys.iter()
                .map(|k| format!("{:?}", k))
                .collect::<Vec<_>>()
                .join("+")
        },
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            id: "default_shortcut".to_string(),
            key_combination: KeyCombination::Single(egui::Key::A),
            action: ShortcutAction::Command("default".to_string()),
            description: "Default shortcut".to_string(),
            category: ShortcutCategory::Custom,
            enabled: true,
            global: false,
        }
    }
}
