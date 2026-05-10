use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub category: String,
    pub genre: String,
    pub rating: Option<f32>,
    pub language: String,
    pub software: String,
    pub notes: String,
    pub custom_fields: HashMap<String, String>,
    pub asset_metadata: HashMap<String, AssetMetadataInfo>,
    pub project_settings: ProjectSettingsMetadata,
    pub export_settings: ExportSettingsMetadata,
    pub collaboration: CollaborationMetadata,
    pub version_control: VersionControlMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadataInfo {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub format: String,
    pub size: u64,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub color_space: Option<String>,
    pub compression: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettingsMetadata {
    pub auto_save: bool,
    pub auto_save_interval: u64,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub thumbnail_size: (u32, u32),
    pub preview_quality: u8,
    pub workspace_layout: String,
    pub ui_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettingsMetadata {
    pub default_format: String,
    pub quality: u8,
    pub resolution: Option<(u32, u32)>,
    pub frame_rate: Option<f32>,
    pub audio_quality: Option<u8>,
    pub compression_settings: HashMap<String, String>,
    pub metadata_inclusion: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMetadata {
    pub collaborators: Vec<CollaboratorInfo>,
    pub permissions: HashMap<String, Vec<String>>,
    pub shared_with: Vec<String>,
    pub owner: String,
    pub created_by: String,
    pub last_modified_by: String,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub status: CollaboratorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaboratorStatus {
    Active,
    Inactive,
    Pending,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
    Admin,
    Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionControlMetadata {
    pub system: String,
    pub repository_url: Option<String>,
    pub branch: String,
    pub commit_hash: String,
    pub last_sync: DateTime<Utc>,
    pub auto_sync: bool,
    pub sync_frequency: u64,
    pub conflict_resolution: String,
    pub ignored_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataValidator {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub field_types: HashMap<String, FieldType>,
    pub validation_rules: HashMap<String, ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    Array,
    Object,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub parameters: HashMap<String, String>,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Range,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSearch {
    pub query: String,
    pub fields: Vec<String>,
    pub filters: Vec<SearchFilter>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataExport {
    pub format: ExportFormat,
    pub fields: Vec<String>,
    pub include_assets: bool,
    pub include_settings: bool,
    pub include_collaboration: bool,
    pub include_version_control: bool,
    pub compression: bool,
    pub encryption: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportFormat {
    Json,
    Toml,
    Yaml,
    Csv,
    Xml,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataImport {
    pub format: ImportFormat,
    pub merge_strategy: MergeStrategy,
    pub overwrite_existing: bool,
    pub validate_on_import: bool,
    pub create_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportFormat {
    Json,
    Toml,
    Yaml,
    Csv,
    Xml,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MergeStrategy {
    Skip,
    Overwrite,
    Merge,
    Prompt,
}

impl ProjectMetadata {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        
        Self {
            id,
            name,
            description: String::new(),
            version: "1.0.0".to_string(),
            author: String::new(),
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            category: "General".to_string(),
            genre: "Uncategorized".to_string(),
            rating: None,
            language: "English".to_string(),
            software: "Tiffiny Studio".to_string(),
            notes: String::new(),
            custom_fields: HashMap::new(),
            asset_metadata: HashMap::new(),
            project_settings: ProjectSettingsMetadata::default(),
            export_settings: ExportSettingsMetadata::default(),
            collaboration: CollaborationMetadata::default(),
            version_control: VersionControlMetadata::default(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = category;
        self
    }

    pub fn with_rating(mut self, rating: f32) -> Self {
        self.rating = Some(rating);
        self
    }

    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    pub fn with_software(mut self, software: String) -> Self {
        self.software = software;
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = notes;
        self
    }

    pub fn add_custom_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
    }

    pub fn get_custom_field(&self, key: &str) -> Option<&String> {
        self.custom_fields.get(key)
    }

    pub fn remove_custom_field(&mut self, key: &str) -> Option<String> {
        self.custom_fields.remove(key)
    }

    pub fn add_asset_metadata(&mut self, asset_id: String, metadata: AssetMetadataInfo) {
        self.asset_metadata.insert(asset_id, metadata);
    }

    pub fn get_asset_metadata(&self, asset_id: &str) -> Option<&AssetMetadataInfo> {
        self.asset_metadata.get(asset_id)
    }

    pub fn remove_asset_metadata(&mut self, asset_id: &str) -> Option<AssetMetadataInfo> {
        self.asset_metadata.remove(asset_id)
    }

    pub fn update_modified_at(&mut self) {
        self.modified_at = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag)
    }

    pub fn get_tag_count(&self) -> usize {
        self.tags.len()
    }

    pub fn get_age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    pub fn get_age_days(&self) -> i64 {
        self.get_age().num_days()
    }

    pub fn is_recent(&self, days: i64) -> bool {
        self.get_age_days() < days
    }

    pub fn get_project_size(&self) -> u64 {
        self.asset_metadata.values().map(|m| m.size).sum()
    }

    pub fn get_asset_count(&self) -> usize {
        self.asset_metadata.len()
    }

    pub fn get_assets_by_type(&self, asset_type: &str) -> Vec<&AssetMetadataInfo> {
        self.asset_metadata
            .values()
            .filter(|m| m.asset_type == asset_type)
            .collect()
    }

    pub fn get_large_assets(&self, size_threshold: u64) -> Vec<&AssetMetadataInfo> {
        self.asset_metadata
            .values()
            .filter(|m| m.size > size_threshold)
            .collect()
    }

    pub fn get_recent_assets(&self, days: i64) -> Vec<&AssetMetadataInfo> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        self.asset_metadata
            .values()
            .filter(|m| m.modified_at > cutoff_date)
            .collect()
    }

    pub fn search_metadata(&self, search: &MetadataSearch) -> Vec<&AssetMetadataInfo> {
        let mut results: Vec<&AssetMetadataInfo> = self.asset_metadata.values().collect();

Apply text search
        if !search.query.is_empty() {
            results = results
                .into_iter()
                .filter(|asset| {
                    self.matches_query(asset, &search.query, &search.fields)
                })
                .collect();
        }

        for filter in &search.filters {
            results = results
                .into_iter()
                .filter(|asset| {
                    self.matches_filter(asset, filter)
                })
                .collect();
        }

        if let Some(sort_field) = &search.sort_by {
            results.sort_by(|a, b| {
                let a_value = self.get_field_value(a, sort_field);
                let b_value = self.get_field_value(b, sort_field);
                
                match search.sort_order {
                    SortOrder::Ascending => a_value.partial_cmp(&b_value).unwrap_or(std::cmp::Ordering::Equal),
                    SortOrder::Descending => b_value.partial_cmp(&a_value).unwrap_or(std::cmp::Ordering::Equal),
                }
            });
        }

        if let Some(offset) = search.offset {
            results = results.into_iter().skip(offset).collect();
        }

        if let Some(limit) = search.limit {
            results = results.into_iter().take(limit).collect();
        }

        results
    }

    fn matches_query(&self, asset: &AssetMetadataInfo, query: &str, fields: &[String]) -> bool {
        let query_lower = query.to_lowercase();
        
        if fields.is_empty() {
            return asset.name.to_lowercase().contains(&query_lower) ||
                   asset.asset_type.to_lowercase().contains(&query_lower) ||
                   asset.format.to_lowercase().contains(&query_lower) ||
                   asset.checksum.to_lowercase().contains(&query_lower) ||
                   asset.color_space.as_ref().map(|s| s.to_lowercase()).unwrap_or_default().contains(&query_lower) ||
                   asset.compression.as_ref().map(|s| s.to_lowercase()).unwrap_or_default().contains(&query_lower) ||
                   asset.metadata.values().any(|v| v.to_lowercase().contains(&query_lower));
        }

        for field in fields {
            if self.field_contains(asset, field, &query_lower) {
                return true;
            }
        }

        false
    }

    fn field_contains(&self, asset: &AssetMetadataInfo, field: &str, query: &str) -> bool {
        match field {
            "name" => asset.name.to_lowercase().contains(query),
            "asset_type" => asset.asset_type.to_lowercase().contains(query),
            "format" => asset.format.to_lowercase().contains(query),
            "checksum" => asset.checksum.to_lowercase().contains(query),
            "color_space" => asset.color_space.as_ref().map(|s| s.to_lowercase()).unwrap_or_default().contains(query),
            "compression" => asset.compression.as_ref().map(|s| s.to_lowercase()).unwrap_or_default().contains(query),
            _ => asset.metadata.get(field).map(|v| v.to_lowercase().contains(query)).unwrap_or(false),
        }
    }

    fn matches_filter(&self, asset: &AssetMetadataInfo, filter: &SearchFilter) -> bool {
        let field_value = self.get_field_value(asset, &filter.field);
        
        match filter.operator {
            FilterOperator::Equals => self.compare_values(&field_value, &filter.value, "=="),
            FilterOperator::NotEquals => !self.compare_values(&field_value, &filter.value, "=="),
            FilterOperator::Contains => field_value.to_lowercase().contains(&filter.value.to_lowercase()),
            FilterOperator::NotContains => !field_value.to_lowercase().contains(&filter.value.to_lowercase()),
            FilterOperator::GreaterThan => self.compare_values(&field_value, &filter.value, ">"),
            FilterOperator::LessThan => self.compare_values(&field_value, &filter.value, "<"),
            FilterOperator::GreaterThanOrEqual => self.compare_values(&field_value, &filter.value, ">="),
            FilterOperator::LessThanOrEqual => self.compare_values(&field_value, &filter.value, "<="),
            FilterOperator::In => filter.value.split(',').any(|v| self.compare_values(&field_value, v.trim(), "==")),
            FilterOperator::NotIn => !filter.value.split(',').any(|v| self.compare_values(&field_value, v.trim(), "==")),
        }
    }

    fn get_field_value(&self, asset: &AssetMetadataInfo, field: &str) -> String {
        match field {
            "name" => asset.name.clone(),
            "asset_type" => asset.asset_type.clone(),
            "format" => asset.format.clone(),
            "size" => asset.size.to_string(),
            "checksum" => asset.checksum.clone(),
            "created_at" => asset.created_at.to_rfc3339(),
            "modified_at" => asset.modified_at.to_rfc3339(),
            "dimensions" => asset.dimensions.map(|d| format!("{}x{}", d.0, d.1)).unwrap_or_default(),
            "duration" => asset.duration.map(|d| d.to_string()).unwrap_or_default(),
            "sample_rate" => asset.sample_rate.map(|s| s.to_string()).unwrap_or_default(),
            "bit_depth" => asset.bit_depth.map(|b| b.to_string()).unwrap_or_default(),
            "channels" => asset.channels.map(|c| c.to_string()).unwrap_or_default(),
            "color_space" => asset.color_space.clone().unwrap_or_default(),
            "compression" => asset.compression.clone().unwrap_or_default(),
            _ => asset.metadata.get(field).cloned().unwrap_or_default(),
        }
    }

    fn compare_values(&self, field_value: &str, filter_value: &str, operator: &str) -> bool {
        match operator {
            "==" => field_value == filter_value,
            ">" => {
                if let (Ok(fv), Ok(fv_filter)) = (field_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    fv > fv_filter
                } else {
                    false
                }
            },
            "<" => {
                if let (Ok(fv), Ok(fv_filter)) = (field_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    fv < fv_filter
                } else {
                    false
                }
            },
            ">=" => {
                if let (Ok(fv), Ok(fv_filter)) = (field_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    fv >= fv_filter
                } else {
                    false
                }
            },
            "<=" => {
                if let (Ok(fv), Ok(fv_filter)) = (field_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    fv <= fv_filter
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    pub fn validate(&self, validator: &MetadataValidator) -> Vec<String> {
        let mut errors = Vec::new();

        for field in &validator.required_fields {
            if !self.has_field(field) {
                errors.push(format!("Required field '{}' is missing", field));
            }
        }

        for (field, rule) in &validator.validation_rules {
            if let Some(field_value) = self.get_field_value_any(field) {
                if !self.validate_field_value(field, field_value, rule) {
                    errors.push(rule.error_message.clone());
                }
            }
        }

        errors
    }

    fn has_field(&self, field: &str) -> bool {
        match field {
            "name" => !self.name.is_empty(),
            "id" => !self.id.is_empty(),
            "version" => !self.version.is_empty(),
            "author" => !self.author.is_empty(),
            "created_at" => true,
            "modified_at" => true,
            _ => self.custom_fields.contains_key(field),
        }
    }

    fn get_field_value_any(&self, field: &str) -> Option<serde_json::Value> {
        match field {
            "name" => Some(serde_json::Value::String(self.name.clone())),
            "id" => Some(serde_json::Value::String(self.id.clone())),
            "version" => Some(serde_json::Value::String(self.version.clone())),
            "author" => Some(serde_json::Value::String(self.author.clone())),
            "rating" => self.rating.map(|r| serde_json::Value::Number(serde_json::Number::from_f64(r as f64))),
            "created_at" => Some(serde_json::Value::String(self.created_at.to_rfc3339())),
            "modified_at" => Some(serde_json::Value::String(self.modified_at.to_rfc3339())),
            _ => self.custom_fields.get(field).map(|v| serde_json::Value::String(v.clone())),
        }
    }

    fn validate_field_value(&self, field: &str, value: &serde_json::Value, rule: &ValidationRule) -> bool {
        match rule.rule_type {
            ValidationType::Required => {
                !value.is_null()
            },
            ValidationType::MinLength => {
                if let Some(min_len) = rule.parameters.get("min_length") {
                    if let (Ok(min_len), Ok(string_val)) = (min_len.parse::<usize>(), value.as_str()) {
                        string_val.len() >= min_len
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ValidationType::MaxLength => {
                if let Some(max_len) = rule.parameters.get("max_length") {
                    if let (Ok(max_len), Ok(string_val)) = (max_len.parse::<usize>(), value.as_str()) {
                        string_val.len() <= max_len
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ValidationType::Pattern => {
                if let Some(pattern) = rule.parameters.get("pattern") {
                    if let (Ok(regex), Ok(string_val)) = (regex::Regex::new(pattern), value.as_str()) {
                        regex.is_match(string_val)
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ValidationType::Range => {
                if let (Some(min), Some(max)) = (rule.parameters.get("min"), rule.parameters.get("max")) {
                    if let (Ok(min_val), Ok(max_val), Ok(num_val)) = (min.parse::<f64>(), max.parse::<f64>(), value.as_f64()) {
                        num_val >= min_val && num_val <= max_val
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            ValidationType::Custom(_) => true,
        }
    }

    pub fn export_metadata(&self, export_config: &MetadataExport) -> Result<String, Box<dyn std::error::Error>> {
        let mut export_data = HashMap::new();

        if export_config.fields.is_empty() || export_config.fields.contains(&"basic".to_string()) {
            export_data.insert("id".to_string(), serde_json::to_value(&self.id));
            export_data.insert("name".to_string(), serde_json::to_value(&self.name));
            export_data.insert("description".to_string(), serde_json::to_value(&self.description));
            export_data.insert("version".to_string(), serde_json::to_value(&self.version));
            export_data.insert("author".to_string(), serde_json::to_value(&self.author));
            export_data.insert("created_at".to_string(), serde_json::to_value(&self.created_at));
            export_data.insert("modified_at".to_string(), serde_json::to_value(&self.modified_at));
            export_data.insert("tags".to_string(), serde_json::to_value(&self.tags));
        }

        if export_config.include_assets {
            export_data.insert("asset_metadata".to_string(), serde_json::to_value(&self.asset_metadata));
        }

        if export_config.include_settings {
            export_data.insert("project_settings".to_string(), serde_json::to_value(&self.project_settings));
        }

        if export_config.include_collaboration {
            export_data.insert("collaboration".to_string(), serde_json::to_value(&self.collaboration));
        }

        if export_config.include_version_control {
            export_data.insert("version_control".to_string(), serde_json::to_value(&self.version_control));
        }

        export_data.insert("custom_fields".to_string(), serde_json::to_value(&self.custom_fields));

        match export_config.format {
            ExportFormat::Json => serde_json::to_string_pretty(&export_data),
            ExportFormat::Toml => toml::to_string_pretty(&export_data)?,
            ExportFormat::Yaml => serde_yaml::to_string(&export_data).map_err(|e| format!("YAML serialization failed: {}", e))?,
            ExportFormat::Csv => self.export_as_csv(&export_data)?,
            ExportFormat::Xml => self.export_as_xml(&export_data)?,
            ExportFormat::Binary => bincode::serialize(&export_data).map(|b| format!("{:?}", b))?,
        }
    }

    fn export_as_csv(&self, data: &HashMap<String, serde_json::Value>) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv_data = Vec::new();
        
        for (key, value) in data {
            csv_data.push(vec![key, self.value_to_string(&value)]);
        }

        let mut csv_string = String::new();
        for row in csv_data {
            csv_string.push_str(&row.join(","));
            csv_string.push('\n');
        }

        Ok(csv_string)
    }

    fn export_as_xml(&self, data: &HashMap<String, serde_json::Value>) -> Result<String, Box<dyn std::error::Error>> {
        let mut xml_string = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project_metadata>\n");

        for (key, value) in data {
            xml_string.push_str(&format!("  <{}>{}</{}>\n", key, self.value_to_xml_string(&value)));
        }

        xml_string.push_str("</project_metadata>");
        Ok(xml_string)
    }

    fn value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(arr) => format!("[{}]", arr.iter().map(|v| self.value_to_string(v)).collect::<Vec<_>>().join(", "))),
            serde_json::Value::Object(obj) => format!("{{{}}}", obj.iter().map(|(k, v)| format!("\"{}\": {}", k, self.value_to_string(v))).collect::<Vec<_>>().join(", "))),
            serde_json::Value::Null => "null".to_string(),
        }
    }

    fn value_to_xml_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("<![CDATA[{}]]>", s),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(arr) => format!("<array>{}</array>", arr.iter().map(|v| self.value_to_xml_string(v)).collect::<Vec<_>>().join("")),
            serde_json::Value::Object(obj) => format!("<object>{}</object>", obj.iter().map(|(k, v)| format!("<{}>{}</{}>", k, self.value_to_xml_string(v))).collect::<Vec<_>>().join("")),
            serde_json::Value::Null => "<null/>".to_string(),
        }
    }

    pub fn import_metadata(&self, import_config: &MetadataImport, data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed_data: serde_json::Value = match import_config.format {
            ImportFormat::Json => serde_json::from_str(data)?,
            ImportFormat::Toml => toml::from_str(data)?,
            ImportFormat::Yaml => serde_yaml::from_str(data)?,
            ImportFormat::Csv => self.parse_csv_to_json(data)?,
            ImportFormat::Xml => self.parse_xml_to_json(data)?,
            ImportFormat::Binary => return Err("Binary import not supported for metadata".into()),
        };

        let metadata: Self = serde_json::from_value(parsed_data)?;

        if !import_config.overwrite_existing {
            self.merge_metadata(&metadata, &import_config.merge_strategy)?;
        }

        if import_config.validate_on_import {
            let validator = MetadataValidator::default();
            let errors = metadata.validate(&validator);
            if !errors.is_empty() {
                return Err(format!("Validation failed: {}", errors.join(", ")).into());
            }
        }

        Ok(metadata)
    }

    fn parse_csv_to_json(&self, csv_data: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut obj = serde_json::Map::new();
        
        for line in csv_data.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                obj.insert(key, serde_json::Value::String(value));
            }
        }

        Ok(serde_json::Value::Object(obj))
    }

    fn parse_xml_to_json(&self, xml_data: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut obj = serde_json::Map::new();
        
        let re = regex::Regex::new(r"<(\w+)>(.*?)</\1>")?;
        
        for caps in re.captures_iter(xml_data) {
            if let Some(tag_match) = caps.get(1) {
                if let Some(value_match) = caps.get(2) {
                    let tag = tag_match.as_str();
                    let value = value_match.as_str();
                    obj.insert(tag.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }

        Ok(serde_json::Value::Object(obj))
    }

    fn merge_metadata(&self, other: &Self, strategy: &MergeStrategy) -> Result<(), Box<dyn std::error::Error>> {
        match strategy {
            MergeStrategy::Skip => {
                Ok(())
            },
            MergeStrategy::Overwrite => {
                Ok(())
            },
            MergeStrategy::Merge => {
                Ok(())
            },
            MergeStrategy::Prompt => {
                Ok(())
            },
        }
    }

    pub fn clone_metadata(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            author: self.author.clone(),
            created_at: self.created_at,
            modified_at: self.modified_at,
            tags: self.tags.clone(),
            category: self.category.clone(),
            genre: self.genre.clone(),
            rating: self.rating,
            language: self.language.clone(),
            software: self.software.clone(),
            notes: self.notes.clone(),
            custom_fields: self.custom_fields.clone(),
            asset_metadata: self.asset_metadata.clone(),
            project_settings: self.project_settings.clone(),
            export_settings: self.export_settings.clone(),
            collaboration: self.collaboration.clone(),
            version_control: self.version_control.clone(),
        }
    }

    pub fn create_snapshot(&self) -> MetadataSnapshot {
        MetadataSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            metadata: self.clone_metadata(),
            changes: Vec::new(),
            snapshot_type: SnapshotType::Manual,
        }
    }

    pub fn get_statistics(&self) -> MetadataStatistics {
        let asset_count = self.asset_metadata.len();
        let total_size = self.asset_metadata.values().map(|m| m.size).sum();
        
        let assets_by_type = self.asset_metadata
            .values()
            .fold(HashMap::new(), |mut map, asset| {
                let count = map.entry(asset.asset_type.clone()).or_insert(0);
                *count += 1;
                map
            });

        let average_asset_size = if asset_count > 0 {
            total_size / asset_count as u64
        } else {
            0
        };

        MetadataStatistics {
            total_assets: asset_count,
            total_size,
            average_asset_size,
            assets_by_type,
            tag_count: self.tags.len(),
            custom_field_count: self.custom_fields.len(),
            created_at: self.created_at,
            modified_at: self.modified_at,
            age_days: self.get_age_days(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSnapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: ProjectMetadata,
    pub changes: Vec<MetadataChange>,
    pub snapshot_type: SnapshotType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChange {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub changed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotType {
    Manual,
    Auto,
    Scheduled,
    BeforeSave,
    AfterSave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataStatistics {
    pub total_assets: usize,
    pub total_size: u64,
    pub average_asset_size: u64,
    pub assets_by_type: HashMap<String, usize>,
    pub tag_count: usize,
    pub custom_field_count: usize,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub age_days: i64,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self::new("default".to_string(), "Default Project".to_string())
    }
}

impl Default for AssetMetadataInfo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Asset".to_string(),
            asset_type: "Binary".to_string(),
            format: "unknown".to_string(),
            size: 0,
            checksum: String::new(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            dimensions: None,
            duration: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            color_space: None,
            compression: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ProjectSettingsMetadata {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval: 300,
            backup_enabled: true,
            backup_count: 5,
            compression_enabled: false,
            compression_level: 6,
            encryption_enabled: false,
            thumbnail_size: (256, 256),
            preview_quality: 80,
            workspace_layout: "default".to_string(),
            ui_settings: HashMap::new(),
        }
    }
}

impl Default for ExportSettingsMetadata {
    fn default() -> Self {
        Self {
            default_format: "json".to_string(),
            quality: 80,
            resolution: None,
            frame_rate: None,
            audio_quality: None,
            compression_settings: HashMap::new(),
            metadata_inclusion: vec!["basic".to_string()],
        }
    }
}

impl Default for CollaborationMetadata {
    fn default() -> Self {
        Self {
            collaborators: Vec::new(),
            permissions: HashMap::new(),
            shared_with: Vec::new(),
            owner: String::new(),
            created_by: String::new(),
            last_modified_by: String::new(),
            access_level: AccessLevel::Owner,
        }
    }
}

impl Default for CollaboratorInfo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Unknown Collaborator".to_string(),
            email: "unknown@example.com".to_string(),
            role: "Viewer".to_string(),
            permissions: vec!["read".to_string()],
            joined_at: Utc::now(),
            last_active: Utc::now(),
            status: CollaboratorStatus::Active,
        }
    }
}

impl Default for VersionControlMetadata {
    fn default() -> Self {
        Self {
            system: "git".to_string(),
            repository_url: None,
            branch: "main".to_string(),
            commit_hash: String::new(),
            last_sync: Utc::now(),
            auto_sync: false,
            sync_frequency: 3600,
            conflict_resolution: "manual".to_string(),
            ignored_files: Vec::new(),
        }
    }
}

impl Default for MetadataValidator {
    fn default() -> Self {
        Self {
            required_fields: vec!["name".to_string(), "id".to_string()],
            optional_fields: vec!["description".to_string(), "tags".to_string()],
            field_types: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            rule_type: ValidationType::Required,
            parameters: HashMap::new(),
            error_message: "Field is required".to_string(),
        }
    }
}

impl Default for MetadataSearch {
    fn default() -> Self {
        Self {
            query: String::new(),
            fields: Vec::new(),
            filters: Vec::new(),
            sort_by: None,
            sort_order: SortOrder::Ascending,
            limit: None,
            offset: None,
        }
    }
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            field: "name".to_string(),
            operator: FilterOperator::Equals,
            value: String::new(),
        }
    }
}

impl Default for MetadataExport {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            fields: vec!["basic".to_string()],
            include_assets: true,
            include_settings: true,
            include_collaboration: false,
            include_version_control: false,
            compression: false,
            encryption: false,
        }
    }
}

impl Default for MetadataImport {
    fn default() -> Self {
        Self {
            format: ImportFormat::Json,
            merge_strategy: MergeStrategy::Skip,
            overwrite_existing: false,
            validate_on_import: true,
            create_backup: false,
        }
    }
}

impl Default for MetadataSnapshot {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            metadata: ProjectMetadata::default(),
            changes: Vec::new(),
            snapshot_type: SnapshotType::Manual,
        }
    }
}

impl Default for MetadataChange {
    fn default() -> Self {
        Self {
            field: String::new(),
            old_value: None,
            new_value: None,
            timestamp: Utc::now(),
            changed_by: "system".to_string(),
        }
    }
}

impl Default for MetadataStatistics {
    fn default() -> Self {
        Self {
            total_assets: 0,
            total_size: 0,
            average_asset_size: 0,
            assets_by_type: HashMap::new(),
            tag_count: 0,
            custom_field_count: 0,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            age_days: 0,
        }
    }
}
