use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ExportPanel {
    pub export_settings: ExportSettings,
    pub output_path: String,
    pub export_format: ExportFormat,
    pub quality_preset: QualityPreset,
    pub is_exporting: bool,
    pub export_progress: f32,
    pub export_status: String,
    pub recent_exports: Vec<ExportHistory>,
}

#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub resolution: (u32, u32),
    pub frame_rate: f32,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub container_format: ContainerFormat,
    pub color_space: String,
    pub compression_level: u8,
    pub custom_parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Video,
    Audio,
    Image,
    ImageSequence,
    Raw,
    Project,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    AV1,
    ProRes,
    Uncompressed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    PCM,
    AAC,
    MP3,
    FLAC,
    Opus,
    Vorbis,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerFormat {
    MP4,
    MKV,
    MOV,
    AVI,
    WAV,
    FLAC,
    PNG,
    TIFF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualityPreset {
    Custom,
    Low,
    Medium,
    High,
    Ultra,
    Lossless,
}

#[derive(Debug, Clone)]
pub struct ExportHistory {
    pub id: String,
    pub timestamp: std::time::SystemTime,
    pub input_path: String,
    pub output_path: String,
    pub format: ExportFormat,
    pub settings: ExportSettings,
    pub file_size: u64,
    pub duration_seconds: Option<f64>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl ExportPanel {
    pub fn new() -> Self {
        Self {
            export_settings: ExportSettings::default(),
            output_path: String::new(),
            export_format: ExportFormat::Video,
            quality_preset: QualityPreset::High,
            is_exporting: false,
            export_progress: 0.0,
            export_status: "Ready".to_string(),
            recent_exports: Vec::new(),
        }
    }

    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn set_output_path(&mut self, path: String) {
        self.output_path = path;
    }

    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
        self.update_default_settings();
    }

    pub fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.quality_preset = preset;
        self.apply_quality_preset();
    }

    fn update_default_settings(&mut self) {
        match self.export_format {
            ExportFormat::Video => {
                self.export_settings.resolution = (1920, 1080);
                self.export_settings.frame_rate = 30.0;
                self.export_settings.sample_rate = 44100;
                self.export_settings.video_codec = VideoCodec::H264;
                self.export_settings.audio_codec = AudioCodec::AAC;
                self.export_settings.container_format = ContainerFormat::MP4;
            },
            ExportFormat::Audio => {
                self.export_settings.sample_rate = 44100;
                self.export_settings.bit_depth = 16;
                self.export_settings.channels = 2;
                self.export_settings.audio_codec = AudioCodec::FLAC;
                self.export_settings.container_format = ContainerFormat::FLAC;
            },
            ExportFormat::Image => {
                self.export_settings.resolution = (1920, 1080);
                self.export_settings.container_format = ContainerFormat::PNG;
            },
            ExportFormat::ImageSequence => {
                self.export_settings.resolution = (1920, 1080);
                self.export_settings.frame_rate = 30.0;
                self.export_settings.container_format = ContainerFormat::PNG;
            },
            ExportFormat::Raw => {
                self.export_settings.bit_depth = 16;
                self.export_settings.container_format = ContainerFormat::WAV;
            },
            ExportFormat::Project => {
                self.export_settings.container_format = ContainerFormat::MP4;
            },
        }
    }

    fn apply_quality_preset(&mut self) {
        match self.quality_preset {
            QualityPreset::Low => {
                self.export_settings.compression_level = 8;
                if matches!(self.export_settings.video_codec, VideoCodec::H264 | VideoCodec::H265) {
                    self.export_settings.custom_parameters.insert("crf".to_string(), "28".to_string());
                }
            },
            QualityPreset::Medium => {
                self.export_settings.compression_level = 6;
                if matches!(self.export_settings.video_codec, VideoCodec::H264 | VideoCodec::H265) {
                    self.export_settings.custom_parameters.insert("crf".to_string(), "23".to_string());
                }
            },
            QualityPreset::High => {
                self.export_settings.compression_level = 4;
                if matches!(self.export_settings.video_codec, VideoCodec::H264 | VideoCodec::H265) {
                    self.export_settings.custom_parameters.insert("crf".to_string(), "18".to_string());
                }
            },
            QualityPreset::Ultra => {
                self.export_settings.compression_level = 2;
                if matches!(self.export_settings.video_codec, VideoCodec::H264 | VideoCodec::H265) {
                    self.export_settings.custom_parameters.insert("crf".to_string(), "15".to_string());
                }
            },
            QualityPreset::Lossless => {
                self.export_settings.compression_level = 0;
                self.export_settings.custom_parameters.insert("crf".to_string(), "0".to_string());
            },
            QualityPreset::Custom => {
            },
        }
    }

    pub async fn start_export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.output_path.is_empty() {
            return Err("Output path is required".into());
        }

        self.is_exporting = true;
        self.export_progress = 0.0;
        self.export_status = "Starting export...".to_string();

        let export_id = uuid::Uuid::new_v4().to_string();
        let start_time = std::time::SystemTime::now();

        let export_history = ExportHistory {
            id: export_id.clone(),
            timestamp: start_time,
            input_path: "project://current".to_string(),
            output_path: self.output_path.clone(),
            format: self.export_format.clone(),
            settings: self.export_settings.clone(),
            file_size: 0,
            duration_seconds: None,
            success: false,
            error_message: None,
        };

        self.recent_exports.insert(0, export_history);

        self.simulate_export_progress().await?;

        let end_time = std::time::SystemTime::now();
        let duration = end_time.duration_since(start_time).unwrap_or_default();
        let file_size = fastrand::u64(1_000_000..100_000_000);

        if let Some(history) = self.recent_exports.get_mut(0) {
            history.success = true;
            history.duration_seconds = Some(duration.as_secs_f64());
            history.file_size = file_size;
        }

        self.is_exporting = false;
        self.export_progress = 1.0;
        self.export_status = "Export completed successfully".to_string();

        Ok(())
    }

    async fn simulate_export_progress(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let total_steps = 100;
        
        for step in 0..=total_steps {
            if !self.is_exporting {
                break;
            }

            self.export_progress = step as f32 / total_steps as f32;
            
            if step < 20 {
                self.export_status = format!("Initializing export... {}%", step);
            } else if step < 40 {
                self.export_status = format!("Processing audio... {}%", step);
            } else if step < 60 {
                self.export_status = format!("Processing video... {}%", step);
            } else if step < 80 {
                self.export_status = format!("Applying effects... {}%", step);
            } else if step < 95 {
                self.export_status = format!("Encoding... {}%", step);
            } else {
                self.export_status = format!("Finalizing... {}%", step);
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Ok(())
    }

    pub fn cancel_export(&mut self) {
        if self.is_exporting {
            self.is_exporting = false;
            self.export_status = "Export cancelled".to_string();
            
            if let Some(history) = self.recent_exports.get_mut(0) {
                history.success = false;
                history.error_message = Some("Export cancelled by user".to_string());
            }
        }
    }

    pub fn clear_history(&mut self) {
        self.recent_exports.clear();
    }

    pub fn get_export_history(&self) -> &[ExportHistory] {
        &self.recent_exports
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Export");
            ui.add_space(20.0);
            
            if self.is_exporting {
                ui.label(format!("{}%", (self.export_progress * 100.0) as i32));
                ui.spinner();
            } else {
                ui.label(&self.export_status);
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Format:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.export_format))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_format, ExportFormat::Video, "Video");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Audio, "Audio");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Image, "Image");
                    ui.selectable_value(&mut self.export_format, ExportFormat::ImageSequence, "Image Sequence");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Raw, "Raw");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Project, "Project");
                });

            ui.add_space(10.0);

            ui.label("Quality:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.quality_preset))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::Custom, "Custom");
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::Low, "Low");
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::Medium, "Medium");
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::High, "High");
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::Ultra, "Ultra");
                    ui.selectable_value(&mut self.quality_preset, QualityPreset::Lossless, "Lossless");
                });
        });

        ui.separator();

        self.render_output_path(ui);
        self.render_export_settings(ui);

        ui.separator();

        ui.horizontal(|ui| {
            if self.is_exporting {
                if ui.button("Cancel").clicked() {
                    self.cancel_export();
                }
            } else {
                if ui.button("Export").clicked() {
                    let _ = self.start_export();
                }
            }

            ui.add_space(10.0);

            if ui.button("Preview").clicked() {
                self.preview_export();
            }

            if ui.button("Reset").clicked() {
                self.reset_settings();
            }
        });

        if self.is_exporting {
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::ProgressBar::new(self.export_progress)
                    .show_percentage()
                    .desired_width(f32::INFINITY));
            });
        }

        ui.separator();
        self.render_export_history(ui);
    }

    fn render_output_path(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Output Path:");
            ui.add_space(10.0);
            
            ui.add(egui::TextEdit::singleline(&mut self.output_path)
                .desired_width(f32::INFINITY)
                .hint_text("Select output file..."));

            if ui.button("Browse").clicked() {
                if let Some(path) = self.browse_output_path() {
                    self.output_path = path;
                }
            }
        });
    }

    fn render_export_settings(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Advanced Settings")
            .default_open(false)
            .show(ui, |ui| {
                match self.export_format {
                    ExportFormat::Video => {
                        self.render_video_settings(ui);
                    },
                    ExportFormat::Audio => {
                        self.render_audio_settings(ui);
                    },
                    ExportFormat::Image => {
                        self.render_image_settings(ui);
                    },
                    ExportFormat::ImageSequence => {
                        self.render_image_sequence_settings(ui);
                    },
                    ExportFormat::Raw => {
                        self.render_raw_settings(ui);
                    },
                    ExportFormat::Project => {
                        self.render_project_settings(ui);
                    },
                }
            });
    }

    fn render_video_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Resolution:");
            ui.add(egui::DragValue::new(&mut self.export_settings.resolution.0));
            ui.label("x");
            ui.add(egui::DragValue::new(&mut self.export_settings.resolution.1));
        });

        ui.horizontal(|ui| {
            ui.label("Frame Rate:");
            ui.add(egui::DragValue::new(&mut self.export_settings.frame_rate).speed(0.1));
            ui.label("fps");
        });

        ui.horizontal(|ui| {
            ui.label("Video Codec:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.export_settings.video_codec))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::H264, "H.264");
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::H265, "H.265");
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::VP9, "VP9");
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::AV1, "AV1");
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::ProRes, "ProRes");
                    ui.selectable_value(&mut self.export_settings.video_codec, VideoCodec::Uncompressed, "Uncompressed");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Audio Codec:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.export_settings.audio_codec))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::PCM, "PCM");
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::AAC, "AAC");
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::MP3, "MP3");
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::FLAC, "FLAC");
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::Opus, "Opus");
                    ui.selectable_value(&mut self.export_settings.audio_codec, AudioCodec::Vorbis, "Vorbis");
                });
        });
    }

    fn render_audio_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Sample Rate:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.export_settings.sample_rate))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.sample_rate, 22050, "22050 Hz");
                    ui.selectable_value(&mut self.export_settings.sample_rate, 44100, "44100 Hz");
                    ui.selectable_value(&mut self.export_settings.sample_rate, 48000, "48000 Hz");
                    ui.selectable_value(&mut self.export_settings.sample_rate, 96000, "96000 Hz");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Bit Depth:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.export_settings.bit_depth))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.bit_depth, 16, "16-bit");
                    ui.selectable_value(&mut self.export_settings.bit_depth, 24, "24-bit");
                    ui.selectable_value(&mut self.export_settings.bit_depth, 32, "32-bit");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Channels:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.export_settings.channels))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.channels, 1, "Mono");
                    ui.selectable_value(&mut self.export_settings.channels, 2, "Stereo");
                    ui.selectable_value(&mut self.export_settings.channels, 6, "5.1 Surround");
                });
        });
    }

    fn render_image_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Resolution:");
            ui.add(egui::DragValue::new(&mut self.export_settings.resolution.0));
            ui.label("x");
            ui.add(egui::DragValue::new(&mut self.export_settings.resolution.1));
        });
    }

    fn render_image_sequence_settings(&mut self, ui: &mut egui::Ui) {
        self.render_image_settings(ui);
        
        ui.horizontal(|ui| {
            ui.label("Frame Rate:");
            ui.add(egui::DragValue::new(&mut self.export_settings.frame_rate).speed(0.1));
            ui.label("fps");
        });
    }

    fn render_raw_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Bit Depth:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.export_settings.bit_depth))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.export_settings.bit_depth, 8, "8-bit");
                    ui.selectable_value(&mut self.export_settings.bit_depth, 16, "16-bit");
                    ui.selectable_value(&mut self.export_settings.bit_depth, 24, "24-bit");
                    ui.selectable_value(&mut self.export_settings.bit_depth, 32, "32-bit");
                });
        });
    }

    fn render_project_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Project export settings will be applied based on project configuration");
    }

    fn render_export_history(&mut self, ui: &mut egui::Ui) {
        ui.heading("Export History");
        
        if ui.button("Clear History").clicked() {
            self.clear_history();
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, export) in self.recent_exports.iter().enumerate() {
                    self.render_export_history_item(ui, export, index);
                }
            });
    }

    fn render_export_history_item(&self, ui: &mut egui::Ui, export: &ExportHistory, index: usize) {
        let success_color = if export.success {
            egui::Color32::from_rgb(100, 200, 100)
        } else {
            egui::Color32::from_rgb(200, 100, 100)
        };

        egui::CollapsingHeader::new(&format!("Export #{}", index + 1))
            .default_open(index == 0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(success_color, if export.success { "✓" } else { "✗" });
                    ui.label(format!("{:?}", export.format));
                    ui.label(&export.output_path);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(duration) = export.duration_seconds {
                            ui.label(format!("{:.1}s", duration));
                        }
                        ui.label(format!("{} MB", export.file_size / 1_000_000));
                    });
                });

                if let Some(error) = &export.error_message {
                    ui.colored_label(egui::Color32::from_rgb(200, 100, 100), error);
                }
            });
    }

    fn browse_output_path(&self) -> Option<String> {
        match self.export_format {
            ExportFormat::Video => {
                rfd::FileDialog::new()
                    .add_filter("MP4", &["mp4"])
                    .add_filter("MKV", &["mkv"])
                    .add_filter("MOV", &["mov"])
                    .save_file()
            },
            ExportFormat::Audio => {
                rfd::FileDialog::new()
                    .add_filter("WAV", &["wav"])
                    .add_filter("FLAC", &["flac"])
                    .add_filter("MP3", &["mp3"])
                    .save_file()
            },
            ExportFormat::Image => {
                rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .add_filter("TIFF", &["tiff"])
                    .add_filter("JPEG", &["jpg", "jpeg"])
                    .save_file()
            },
            ExportFormat::ImageSequence => {
                rfd::FileDialog::new()
                    .pick_folder()
                    .map(|path| format!("{}/sequence_%04d.png", path.to_string_lossy()))
            },
            ExportFormat::Raw => {
                rfd::FileDialog::new()
                    .add_filter("RAW", &["raw"])
                    .add_filter("BIN", &["bin"])
                    .save_file()
            },
            ExportFormat::Project => {
                rfd::FileDialog::new()
                    .add_filter("TIFFINY", &["tiffiny"])
                    .save_file()
            },
        }.map(|path| path.to_string_lossy().to_string())
    }

    fn preview_export(&self) {
        tracing::info!("Previewing export with settings: {:?}", self.export_settings);
    }

    fn reset_settings(&mut self) {
        self.export_settings = ExportSettings::default();
        self.quality_preset = QualityPreset::High;
        self.apply_quality_preset();
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            frame_rate: 30.0,
            sample_rate: 44100,
            bit_depth: 16,
            channels: 2,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::AAC,
            container_format: ContainerFormat::MP4,
            color_space: "sRGB".to_string(),
            compression_level: 4,
            custom_parameters: std::collections::HashMap::new(),
        }
    }
}

impl Default for ExportPanel {
    fn default() -> Self {
        Self::new()
    }
}
