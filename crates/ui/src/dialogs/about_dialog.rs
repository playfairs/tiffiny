use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct AboutDialog {
    pub id: String,
    pub title: String,
    pub app_name: String,
    pub version: String,
    pub build_date: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub website: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub credits: Vec<Credit>,
    pub visible: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct Credit {
    pub name: String,
    pub role: String,
    pub contribution: String,
}

impl AboutDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "About Tiffiny Studio".to_string(),
            app_name: "Tiffiny Studio".to_string(),
            version: "1.0.0".to_string(),
            build_date: "2024".to_string(),
            description: "Professional multimedia databending and media reinterpretation studio".to_string(),
            authors: vec![
                "Tiffiny Studio Development Team".to_string(),
            ],
            license: "MIT".to_string(),
            website: Some("https://tiffiny-studio.com".to_string()),
            repository: Some("https://github.com/tiffiny-studio/tiffiny".to_string()),
            documentation: Some("https://docs.tiffiny-studio.com".to_string()),
            credits: vec![
                Credit {
                    name: "Creative Director".to_string(),
                    role: "Lead Developer".to_string(),
                    contribution: "Architecture and system design".to_string(),
                },
                Credit {
                    name: "Audio Engine Team".to_string(),
                    role: "Audio Specialists".to_string(),
                    contribution: "Audio processing and effects".to_string(),
                },
                Credit {
                    name: "Graphics Team".to_string(),
                    role: "Graphics Specialists".to_string(),
                    contribution: "GPU rendering and visualization".to_string(),
                },
                Credit {
                    name: "UI/UX Team".to_string(),
                    role: "Interface Designers".to_string(),
                    contribution: "User interface and experience design".to_string(),
                },
                Credit {
                    name: "Community Contributors".to_string(),
                    role: "Beta Testers".to_string(),
                    contribution: "Testing and feedback".to_string(),
                },
            ],
            visible: false,
            on_close: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn build_date(mut self, date: impl Into<String>) -> Self {
        self.build_date = date.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.authors = authors;
        self
    }

    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = license.into();
        self
    }

    pub fn website(mut self, website: Option<String>) -> Self {
        self.website = website;
        self
    }

    pub fn repository(mut self, repository: Option<String>) -> Self {
        self.repository = repository;
        self
    }

    pub fn documentation(mut self, documentation: Option<String>) -> Self {
        self.documentation = documentation;
        self
    }

    pub fn credits(mut self, credits: Vec<Credit>) -> Self {
        self.credits = credits;
        self
    }

    pub fn on_close(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(callback));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(600.0, 500.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
                    .rounding(8.0);

                frame.show(ui, |ui| {
Title bar
                    self.render_title_bar(ui, &mut closed);

                    ui.separator();

                    self.render_content(ui);
                });
            });

        if closed {
            self.hide();
            if let Some(callback) = &self.on_close {
                callback();
            }
        }

        closed
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_content(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("🎨");
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.heading(&self.app_name);
                        ui.label(&self.version);
                        ui.label(format!("Build: {}", self.build_date));
                    });
                });

                ui.separator();

                ui.heading("Description");
                ui.label(&self.description);

                ui.add_space(20.0);

                ui.heading("Links");
                
                if let Some(website) = &self.website {
                    ui.horizontal(|ui| {
                        ui.label("Website:");
                        if ui.hyperlink_to(website).clicked() {
                        }
                        ui.colored_label(ui.visuals().hyperlink_color(), website);
                    });
                }

                if let Some(repository) = &self.repository {
                    ui.horizontal(|ui| {
                        ui.label("Repository:");
                        if ui.hyperlink_to(repository).clicked() {
                        }
                        ui.colored_label(ui.visuals().hyperlink_color(), repository);
                    });
                }

                if let Some(documentation) = &self.documentation {
                    ui.horizontal(|ui| {
                        ui.label("Documentation:");
                        if ui.hyperlink_to(documentation).clicked() {
                        }
                        ui.colored_label(ui.visuals().hyperlink_color(), documentation);
                    });
                }

                ui.add_space(20.0);

                ui.heading("License");
                ui.label(&self.license);

                ui.add_space(20.0);

                ui.heading("Authors");
                for author in &self.authors {
                    ui.label(author);
                }

                ui.add_space(20.0);

                if !self.credits.is_empty() {
                    ui.heading("Credits");
                    egui::CollapsingHeader::new("Development Team")
                        .default_open(false)
                        .show(ui, |ui| {
                            for credit in &self.credits {
                                ui.horizontal(|ui| {
                                    ui.label(&credit.name);
                                    ui.label(":");
                                    ui.colored_label(
                                        ui.visuals().text_color().multiply(0.7),
                                        &credit.role
                                    );
                                    ui.add_space(10.0);
                                    ui.colored_label(
                                        ui.visuals().text_color().multiply(0.7),
                                        &credit.contribution
                                    );
                                });
                            }
                        });
                }
            });
    }
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new("default_about_dialog".to_string())
    }
}

impl Default for Credit {
    fn default() -> Self {
        Self {
            name: "Default Credit".to_string(),
            role: "Default Role".to_string(),
            contribution: "Default contribution".to_string(),
        }
    }
}
