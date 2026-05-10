use std::collections::HashMap;
use crate::project::{Project, Asset, ProjectMetadata, AssetMetadata, AssetType, ProjectState};

#[derive(Debug, Clone)]
pub struct ProjectValidator {
    pub id: String,
    pub name: String,
    pub rules: HashMap<String, ValidationRule>,
    pub settings: ValidatorSettings,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub field: String,
    pub rule_type: ValidationType,
    pub parameters: ValidationParameters,
    pub severity: ValidationSeverity,
    pub error_message: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Range,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ValidationParameters {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<String>>,
    pub custom_params: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ValidatorSettings {
    pub strict_mode: bool,
    pub validate_assets: bool,
    pub validate_metadata: bool,
    pub validate_paths: bool,
    pub validate_file_sizes: bool,
    pub max_warnings: usize,
    pub stop_on_first_error: bool,
    pub custom_validators: HashMap<String, CustomValidator>,
}

#[derive(Debug, Clone)]
pub struct CustomValidator {
    pub name: String,
    pub validator_type: String,
    pub parameters: ValidationParameters,
    pub implementation: String,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Vec<ValidationInfo>,
    pub summary: ValidationSummary,
    pub validation_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub value: String,
    pub rule_name: String,
    pub message: String,
    pub severity: ValidationSeverity,
    pub context: ValidationContext,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub value: String,
    pub rule_name: String,
    pub message: String,
    pub context: ValidationContext,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationInfo {
    pub field: String,
    pub value: String,
    pub message: String,
    pub context: ValidationContext,
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub project_id: String,
    pub asset_id: Option<String>,
    pub validation_phase: ValidationPhase,
    pub additional_data: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationPhase {
    Project,
    Assets,
    Metadata,
    Paths,
    FileSizes,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_rules: usize,
    pub executed_rules: usize,
    pub errors_count: usize,
    pub warnings_count: usize,
    pub info_count: usize,
    pub validation_level: ValidationLevel,
    pub critical_errors: Vec<String>,
    pub performance_metrics: ValidationMetrics,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    Valid,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ValidationMetrics {
    pub validation_time: std::time::Duration,
    pub rules_executed: usize,
    pub assets_validated: usize,
    pub paths_checked: usize,
    pub file_sizes_checked: usize,
    pub memory_usage: u64,
}

impl ProjectValidator {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            rules: Self::create_default_rules(),
            settings: ValidatorSettings::default(),
        }
    }

    pub fn with_settings(mut self, settings: ValidatorSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.insert(rule.name.clone(), rule);
        self
    }

    pub fn remove_rule(&mut self, rule_name: &str) -> bool {
        self.rules.remove(rule_name).is_some()
    }

    pub fn get_rule(&self, rule_name: &str) -> Option<&ValidationRule> {
        self.rules.get(rule_name)
    }

    pub fn enable_rule(&mut self, rule_name: &str) -> bool {
        if let Some(rule) = self.rules.get_mut(rule_name) {
            rule.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_rule(&mut self, rule_name: &str) -> bool {
        if let Some(rule) = self.rules.get_mut(rule_name) {
            rule.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn validate(&self, project: &Project) -> ValidationResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

Validate project fields
        if self.settings.validate_metadata {
            self.validate_project_fields(project, &mut errors, &mut warnings, &mut info);
        }

        if self.settings.validate_assets {
            self.validate_assets(project, &mut errors, &mut warnings, &mut info);
        }

        if self.settings.validate_paths {
            self.validate_paths(project, &mut errors, &mut warnings, &mut info);
        }

        if self.settings.validate_file_sizes {
            self.validate_file_sizes(project, &mut errors, &mut warnings, &mut info);
        }

        self.apply_custom_validators(project, &mut errors, &mut warnings, &mut info);

        let validation_time = start_time.elapsed();
        let summary = self.create_summary(&errors, &warnings, &info, &validation_time);

        ValidationResult {
            success: errors.is_empty(),
            errors,
            warnings,
            info,
            summary,
            validation_time,
        }
    }

    fn validate_project_fields(&self, project: &Project, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        let context = ValidationContext {
            project_id: project.id.clone(),
            asset_id: None,
            validation_phase: ValidationPhase::Project,
            additional_data: HashMap::new(),
        };

        self.validate_required_field("name", &project.name, &context, errors);
        self.validate_required_field("id", &project.id, &context, errors);

        if let Some(rule) = self.rules.get("name_length") {
            self.validate_length_rule("name", &project.name, rule, &context, warnings);
        }

        if let Some(rule) = self.rules.get("version_format") {
            self.validate_version_format(&project.version, rule, &context, warnings);
        }

        if let Some(rule) = self.rules.get("creation_date") {
            self.validate_date_field("created_at", &project.created_at, rule, &context, warnings);
        }
    }

    fn validate_assets(&self, project: &Project, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        for (asset_id, asset) in &project.assets {
            let context = ValidationContext {
                project_id: project.id.clone(),
                asset_id: Some(asset_id.clone()),
                validation_phase: ValidationPhase::Assets,
                additional_data: HashMap::new(),
            };

            self.validate_required_field("asset_name", &asset.name, &context, errors);

            if let Some(rule) = self.rules.get("asset_path") {
                self.validate_asset_path(&asset.path, rule, &context, warnings);
            self.validate_file_exists(&asset.path, &context, warnings);
            self.validate_file_extension(&asset.path, &context, warnings);
            self.validate_file_size(&asset.size, &context, warnings);
            self.validate_asset_type(&asset.asset_type, &context, warnings);
            self.validate_asset_metadata(&asset.metadata, &context, warnings);
            }

            if let Some(rule) = self.rules.get("asset_timestamps") {
                self.validate_timestamps(&asset.created_at, &asset.modified_at, rule, &context, warnings);
            }
        }
    }

    fn validate_paths(&self, project: &Project, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        let context = ValidationContext {
            project_id: project.id.clone(),
            asset_id: None,
            validation_phase: ValidationPhase::Paths,
            additional_data: HashMap::new(),
        };

        let mut seen_paths = std::collections::HashSet::new();
        for (asset_id, asset) in &project.assets {
            if seen_paths.contains(&asset.path) {
                errors.push(ValidationError {
                    field: "asset_path".to_string(),
                    value: asset.path.to_string_lossy().to_string(),
                    rule_name: "duplicate_paths".to_string(),
                    message: format!("Duplicate asset path: {:?}", asset.path),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
            } else {
                seen_paths.insert(asset.path.clone());
            }
        }

        for (asset_id, asset) in &project.assets {
            let path_str = asset.path.to_string_lossy();
            if self.contains_invalid_characters(&path_str) {
                warnings.push(ValidationError {
                    field: "asset_path".to_string(),
                    value: path_str,
                    rule_name: "invalid_characters".to_string(),
                    message: "Path contains potentially invalid characters".to_string(),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }

        for (asset_id, asset) in &project.assets {
            if let Some(rule) = self.rules.get("path_depth") {
                self.validate_path_depth(&asset.path, rule, &context, warnings);
            }
        }
    }

    fn validate_file_sizes(&self, project: &Project, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        let context = ValidationContext {
            project_id: project.id.clone(),
            asset_id: None,
            validation_phase: ValidationPhase::FileSizes,
            additional_data: HashMap::new(),
        };

        for (asset_id, asset) in &project.assets {
            if let Some(rule) = self.rules.get("file_size") {
                self.validate_file_size(&asset.size, rule, &context, warnings);
            }
        }

        let total_size: u64 = project.assets.values().map(|a| a.size).sum();
        if total_size > 10 * 1024 * 1024 * 1024 {
            warnings.push(ValidationError {
                field: "total_size".to_string(),
                value: total_size.to_string(),
                rule_name: "large_project".to_string(),
                message: "Project size is very large".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }
    }

    fn apply_custom_validators(&self, project: &Project, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        for (validator_name, validator) in &self.settings.custom_validators {
            let context = ValidationContext {
                project_id: project.id.clone(),
                asset_id: None,
                validation_phase: ValidationPhase::Custom,
                additional_data: HashMap::new(),
            };

            match validator.validator_type.as_str() {
                "regex" => self.apply_regex_validator(project, validator, &context, errors, warnings, info),
                "custom_function" => self.apply_custom_function_validator(project, validator, &context, errors, warnings, info),
                _ => {
                    info.push(ValidationInfo {
                        field: "custom_validator".to_string(),
                        value: validator_name.clone(),
                        message: format!("Unknown validator type: {}", validator.validator_type),
                        context: context.clone(),
                    });
                }
            }
        }
    }

    fn validate_required_field(&self, field_name: &str, value: &str, context: &ValidationContext, errors: &mut Vec<ValidationError>) {
        if value.is_empty() {
            errors.push(ValidationError {
                field: field_name.to_string(),
                value: value.to_string(),
                rule_name: "required_field".to_string(),
                message: format!("Field '{}' is required", field_name),
                severity: ValidationSeverity::Error,
                context: context.clone(),
            });
        }
    }

    fn validate_length_rule(&self, field_name: &str, value: &str, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        let length = value.len();
        
        if let Some(min_length) = rule.parameters.min_length {
            if length < min_length {
                warnings.push(ValidationError {
                    field: field_name.to_string(),
                    value: value.to_string(),
                    rule_name: rule.name.clone(),
                    message: format!("Field '{}' is too short (min: {})", field_name, min_length),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }

        if let Some(max_length) = rule.parameters.max_length {
            if length > max_length {
                warnings.push(ValidationError {
                    field: field_name.to_string(),
                    value: value.to_string(),
                    rule_name: rule.name.clone(),
                    message: format!("Field '{}' is too long (max: {})", field_name, max_length),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }
    }

    fn validate_version_format(&self, version: &str, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if let Some(pattern) = &rule.parameters.pattern {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(version) {
                    warnings.push(ValidationError {
                        field: "version".to_string(),
                        value: version.to_string(),
                        rule_name: rule.name.clone(),
                        message: "Version format is invalid".to_string(),
                        severity: ValidationSeverity::Warning,
                        context: context.clone(),
                    });
                }
            }
        }
    }

    fn validate_date_field(&self, field_name: &str, date: &chrono::DateTime<chrono::Utc>, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        let now = chrono::Utc::now();
        
        if let Some(min_value) = rule.parameters.min_value {
            let min_date = now - chrono::Duration::days(min_value as i64);
            if *date < min_date {
                warnings.push(ValidationError {
                    field: field_name.to_string(),
                    value: date.to_rfc3339(),
                    rule_name: rule.name.clone(),
                    message: format!("Date '{}' is too old", field_name),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }

        if let Some(max_value) = rule.parameters.max_value {
            let max_date = now + chrono::Duration::days(max_value as i64);
            if *date > max_date {
                warnings.push(ValidationError {
                    field: field_name.to_string(),
                    value: date.to_rfc3339(),
                    rule_name: rule.name.clone(),
                    message: format!("Date '{}' is in the future", field_name),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }
    }

    fn validate_asset_path(&self, path: &std::path::Path, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if let Some(allowed_values) = &rule.parameters.allowed_values {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy();
                if !allowed_values.contains(&extension_str) {
                    warnings.push(ValidationError {
                        field: "asset_path".to_string(),
                        value: path.to_string_lossy().to_string(),
                        rule_name: rule.name.clone(),
                        message: format!("File extension '{}' is not allowed", extension_str),
                        severity: ValidationSeverity::Warning,
                        context: context.clone(),
                    });
                }
            }
        }
    }

    fn validate_file_exists(&self, path: &std::path::Path, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if !path.exists() {
            warnings.push(ValidationError {
                field: "asset_path".to_string(),
                value: path.to_string_lossy().to_string(),
                rule_name: "file_exists".to_string(),
                message: "File does not exist".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }
    }

    fn validate_file_extension(&self, path: &std::path::Path, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if path.extension().is_none() {
            warnings.push(ValidationError {
                field: "asset_path".to_string(),
                value: path.to_string_lossy().to_string(),
                rule_name: "file_extension".to_string(),
                message: "File has no extension".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }
    }

    fn validate_file_size(&self, size: u64, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if let Some(min_value) = rule.parameters.min_value {
            if size < min_value as u64 {
                warnings.push(ValidationError {
                    field: "file_size".to_string(),
                    value: size.to_string(),
                    rule_name: rule.name.clone(),
                    message: format!("File size is too small (min: {} bytes)", min_value),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }

        if let Some(max_value) = rule.parameters.max_value {
            if size > max_value as u64 {
                warnings.push(ValidationError {
                    field: "file_size".to_string(),
                    value: size.to_string(),
                    rule_name: rule.name.clone(),
                    message: format!("File size is too large (max: {} bytes)", max_value),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }
    }

    fn validate_asset_type(&self, asset_type: &AssetType, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        let supported_types = vec![
            AssetType::Image,
            AssetType::Video,
            AssetType::Audio,
            AssetType::Text,
            AssetType::Binary,
        ];

        if !supported_types.contains(asset_type) {
            warnings.push(ValidationError {
                field: "asset_type".to_string(),
                value: format!("{:?}", asset_type),
                rule_name: "asset_type".to_string(),
                message: "Asset type is not supported".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }
    }

    fn validate_asset_metadata(&self, metadata: &AssetMetadata, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if metadata.format.is_empty() {
            warnings.push(ValidationError {
                field: "metadata.format".to_string(),
                value: metadata.format.clone(),
                rule_name: "metadata_format".to_string(),
                message: "Asset format is empty".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }

        if metadata.format.to_lowercase().contains("image") {
            if let Some((width, height)) = metadata.dimensions {
                if width == 0 || height == 0 {
                    warnings.push(ValidationError {
                        field: "metadata.dimensions".to_string(),
                        value: format!("{}x{}", width, height),
                        rule_name: "image_dimensions".to_string(),
                        message: "Image dimensions cannot be zero".to_string(),
                        severity: ValidationSeverity::Error,
                        context: context.clone(),
                    });
                }
            }
        }

        if metadata.format.to_lowercase().contains("video") || metadata.format.to_lowercase().contains("audio") {
            if let Some(duration) = metadata.duration {
                if duration == std::time::Duration::from_secs(0) {
                    warnings.push(ValidationError {
                        field: "metadata.duration".to_string(),
                        value: duration.as_secs().to_string(),
                        rule_name: "media_duration".to_string(),
                        message: "Media duration cannot be zero".to_string(),
                        severity: ValidationSeverity::Warning,
                        context: context.clone(),
                    });
                }
            }
        }
    }

    fn validate_timestamps(&self, created_at: &chrono::DateTime<chrono::Utc>, modified_at: &chrono::DateTime<chrono::Utc>, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if *modified_at < *created_at {
            warnings.push(ValidationError {
                field: "timestamps".to_string(),
                value: format!("created: {}, modified: {}", created_at.to_rfc3339(), modified_at.to_rfc3339()),
                rule_name: rule.name.clone(),
                message: "Modified timestamp is earlier than creation timestamp".to_string(),
                severity: ValidationSeverity::Warning,
                context: context.clone(),
            });
        }
    }

    fn validate_path_depth(&self, path: &std::path::Path, rule: &ValidationRule, context: &ValidationContext, warnings: &mut Vec<ValidationError>) {
        if let Some(max_depth) = rule.parameters.max_value {
            let depth = path.components().count();
            if depth > max_depth as usize {
                warnings.push(ValidationError {
                    field: "path_depth".to_string(),
                    value: depth.to_string(),
                    rule_name: rule.name.clone(),
                    message: format!("Path depth is too deep (max: {})", max_depth),
                    severity: ValidationSeverity::Warning,
                    context: context.clone(),
                });
            }
        }
    }

    fn contains_invalid_characters(&self, path_str: &str) -> bool {
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        path_str.chars().any(|c| invalid_chars.contains(&c))
    }

    fn apply_regex_validator(&self, project: &Project, validator: &CustomValidator, context: &ValidationContext, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        if let Some(pattern) = &validator.parameters.pattern {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(&project.name) {
                    errors.push(ValidationError {
                        field: "project_name".to_string(),
                        value: project.name.clone(),
                        rule_name: validator.name.clone(),
                        message: "Project name does not match required pattern".to_string(),
                        severity: ValidationSeverity::Error,
                        context: context.clone(),
                    });
                }
            }
        }
    }

    fn apply_custom_function_validator(&self, project: &Project, validator: &CustomValidator, context: &ValidationContext, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>, info: &mut Vec<ValidationInfo>) {
        info.push(ValidationInfo {
            field: "custom_validator".to_string(),
            value: validator.implementation.clone(),
            message: format!("Custom validator '{}' would be applied", validator.name),
            context: context.clone(),
        });
    }

    fn create_summary(&self, errors: &[ValidationError], warnings: &[ValidationError], info: &[ValidationInfo], validation_time: &std::time::Duration) -> ValidationSummary {
        let total_rules = self.rules.len();
        let executed_rules = total_rules;In a real implementation, track which rules were actually executed
        
        let errors_count = errors.len();
        let warnings_count = warnings.len();
        let info_count = info.len();

        let validation_level = if errors_count > 0 {
            if self.settings.strict_mode {
                ValidationLevel::Critical
            } else {
                ValidationLevel::Error
            }
        } else if warnings_count > 0 {
            ValidationLevel::Warning
        } else {
            ValidationLevel::Valid
        };

        let critical_errors: Vec<String> = errors.iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .map(|e| e.message.clone())
            .collect();

        let performance_metrics = ValidationMetrics {
            validation_time: *validation_time,
            rules_executed: executed_rules,
            assets_validated: if self.settings.validate_assets { self.rules.len() } else { 0 },
            paths_checked: if self.settings.validate_paths { self.rules.len() } else { 0 },
            file_sizes_checked: if self.settings.validate_file_sizes { self.rules.len() } else { 0 },
            memory_usage: 0,Would need to track actual memory usage
        };

        ValidationSummary {
            total_rules,
            executed_rules,
            errors_count,
            warnings_count,
            info_count,
            validation_level,
            critical_errors,
            performance_metrics,
        }
    }

    pub fn create_default_rules() -> HashMap<String, ValidationRule> {
        let mut rules = HashMap::new();

Project name validation
        rules.insert("name_length".to_string(), ValidationRule {
            name: "name_length".to_string(),
            field: "name".to_string(),
            rule_type: ValidationType::MinLength,
            parameters: ValidationParameters {
                min_length: Some(1),
                max_length: Some(255),
                pattern: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "Project name length validation".to_string(),
            enabled: true,
        });

Version format validation
        rules.insert("version_format".to_string(), ValidationRule {
            name: "version_format".to_string(),
            field: "version".to_string(),
            rule_type: ValidationType::Pattern,
            parameters: ValidationParameters {
                min_length: None,
                max_length: None,
                pattern: Some(r"^\d+\.\d+\.\d+$".to_string()),
                min_value: None,
                max_value: None,
                allowed_values: None,
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "Version format validation".to_string(),
            enabled: true,
        });

Asset path validation
        rules.insert("asset_path".to_string(), ValidationRule {
            name: "asset_path".to_string(),
            field: "asset_path".to_string(),
            rule_type: ValidationType::Custom("path_validation".to_string()),
            parameters: ValidationParameters {
                min_length: None,
                max_length: None,
                pattern: None,
                min_value: None,
                max_value: None,
                allowed_values: Some(vec![
                    "png".to_string(), "jpg".to_string(), "jpeg".to_string(), "gif".to_string(),
                    "mp4".to_string(), "avi".to_string(), "mov".to_string(),
                    "wav".to_string(), "mp3".to_string(), "flac".to_string(),
                    "txt".to_string(), "md".to_string(),
                ]),
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "Asset path validation".to_string(),
            enabled: true,
        });

File size validation
        rules.insert("file_size".to_string(), ValidationRule {
            name: "file_size".to_string(),
            field: "file_size".to_string(),
            rule_type: ValidationType::Range,
            parameters: ValidationParameters {
                min_length: None,
                max_length: None,
                pattern: None,
                min_value: Some(1.0),
                max_value: Some(1024.0 * 1024.0 * 1024.0),1GB
                allowed_values: None,
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "File size validation".to_string(),
            enabled: true,
        });

Path depth validation
        rules.insert("path_depth".to_string(), ValidationRule {
            name: "path_depth".to_string(),
            field: "path_depth".to_string(),
            rule_type: ValidationType::Range,
            parameters: ValidationParameters {
                min_length: None,
                max_length: None,
                pattern: None,
                min_value: Some(1.0),
                max_value: Some(10.0),
                allowed_values: None,
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "Path depth validation".to_string(),
            enabled: true,
        });

Asset timestamps validation
        rules.insert("asset_timestamps".to_string(), ValidationRule {
            name: "asset_timestamps".to_string(),
            field: "timestamps".to_string(),
            rule_type: ValidationType::Custom("timestamp_validation".to_string()),
            parameters: ValidationParameters {
                min_length: None,
                max_length: None,
                pattern: None,
                min_value: None,
                max_value: None,
                allowed_values: None,
                custom_params: HashMap::new(),
            },
            severity: ValidationSeverity::Warning,
            error_message: "Asset timestamp validation".to_string(),
            enabled: true,
        });

        rules
    }

    pub fn get_rules_summary(&self) -> RulesSummary {
        let total_rules = self.rules.len();
        let enabled_rules = self.rules.values().filter(|r| r.enabled).count();
        let rules_by_type = self.rules.values()
            .fold(HashMap::new(), |mut map, rule| {
                let rule_type = format!("{:?}", rule.rule_type);
                let count = map.entry(rule_type).or_insert(0);
                *count += 1;
                map
            });

        RulesSummary {
            total_rules,
            enabled_rules,
            rules_by_type,
            custom_validators: self.settings.custom_validators.clone(),
        }
    }

    pub fn export_rules(&self) -> Result<String, Box<dyn std::error::Error>> {
        serde_json::to_string_pretty(&self.rules)
    }

    pub fn import_rules(&self, rules_json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let imported_rules: HashMap<String, ValidationRule> = serde_json::from_str(rules_json)?;
        self.rules = imported_rules;
        Ok(())
    }

    pub fn reset_rules(&mut self) {
        self.rules = Self::create_default_rules();
    }

    pub fn clone_validator(&self) -> ProjectValidator {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", self.name),
            rules: self.rules.clone(),
            settings: self.settings.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RulesSummary {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub rules_by_type: HashMap<String, usize>,
    pub custom_validators: HashMap<String, CustomValidator>,
}

impl Default for ProjectValidator {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Validator".to_string(),
        )
    }
}

impl Default for ValidatorSettings {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate_assets: true,
            validate_metadata: true,
            validate_paths: true,
            validate_file_sizes: true,
            max_warnings: 50,
            stop_on_first_error: false,
            custom_validators: HashMap::new(),
        }
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            field: "default".to_string(),
            rule_type: ValidationType::Required,
            parameters: ValidationParameters::default(),
            severity: ValidationSeverity::Warning,
            error_message: "Default validation rule".to_string(),
            enabled: true,
        }
    }
}

impl Default for ValidationParameters {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
            custom_params: HashMap::new(),
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            success: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            summary: ValidationSummary::default(),
            validation_time: std::time::Duration::from_millis(0),
        }
    }
}

impl Default for ValidationError {
    fn default() -> Self {
        Self {
            field: "unknown".to_string(),
            value: "unknown".to_string(),
            rule_name: "unknown".to_string(),
            message: "Validation error".to_string(),
            severity: ValidationSeverity::Warning,
            context: ValidationContext::default(),
        }
    }
}

impl Default for ValidationWarning {
    fn default() -> Self {
        Self {
            field: "unknown".to_string(),
            value: "unknown".to_string(),
            rule_name: "unknown".to_string(),
            message: "Validation warning".to_string(),
            context: ValidationContext::default(),
            suggestion: None,
        }
    }
}

impl Default for ValidationInfo {
    fn default() -> Self {
        Self {
            field: "unknown".to_string(),
            value: "unknown".to_string(),
            message: "Validation info".to_string(),
            context: ValidationContext::default(),
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            project_id: "unknown".to_string(),
            asset_id: None,
            validation_phase: ValidationPhase::Project,
            additional_data: HashMap::new(),
        }
    }
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self {
            total_rules: 0,
            executed_rules: 0,
            errors_count: 0,
            warnings_count: 0,
            info_count: 0,
            validation_level: ValidationLevel::Valid,
            critical_errors: Vec::new(),
            performance_metrics: ValidationMetrics::default(),
        }
    }
}

impl Default for ValidationMetrics {
    fn default() -> Self {
        Self {
            validation_time: std::time::Duration::from_millis(0),
            rules_executed: 0,
            assets_validated: 0,
            paths_checked: 0,
            file_sizes_checked: 0,
            memory_usage: 0,
        }
    }
}

impl Default for RulesSummary {
    fn default() -> Self {
        Self {
            total_rules: 0,
            enabled_rules: 0,
            rules_by_type: HashMap::new(),
            custom_validators: HashMap::new(),
        }
    }
}

impl Default for CustomValidator {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            validator_type: "regex".to_string(),
            parameters: ValidationParameters::default(),
            implementation: "default_validator".to_string(),
        }
    }
}
