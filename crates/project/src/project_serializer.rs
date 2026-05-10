use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::project::{Project, Asset, ProjectSettings, ProjectMetadata, AssetMetadata, AssetType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationFormat {
    pub version: String,
    pub format_type: FormatType,
    pub compression: Option<CompressionInfo>,
    pub encryption: Option<EncryptionInfo>,
    pub metadata: SerializationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FormatType {
    Json,
    Toml,
    Binary,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub algorithm: CompressionAlgorithm,
    pub level: u8,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Lz4,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: EncryptionAlgorithm,
    pub key_id: Option<String>,
    pub iv: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    None,
    Aes256,
    ChaCha20,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub software_version: String,
    pub checksum: String,
    pub schema_version: String,
    pub custom_properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ProjectSerializer {
    pub id: String,
    pub name: String,
    pub format: SerializationFormat,
    pub settings: SerializerSettings,
}

#[derive(Debug, Clone)]
pub struct SerializerSettings {
    pub pretty_print: bool,
    pub include_metadata: bool,
    pub include_assets: bool,
    pub include_binary_data: bool,
    pub validate_after_serialization: bool,
    pub max_binary_size: u64,
    pub chunk_size: usize,
}

#[derive(Debug, Clone)]
pub struct DeserializationResult {
    pub success: bool,
    pub project: Option<Project>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub metadata: DeserializationMetadata,
}

#[derive(Debug, Clone)]
pub struct DeserializationMetadata {
    pub format_detected: FormatType,
    pub version_detected: String,
    pub compression_detected: Option<CompressionAlgorithm>,
    pub encryption_detected: Option<EncryptionAlgorithm>,
    pub schema_version: String,
    pub custom_properties: HashMap<String, String>,
}

impl ProjectSerializer {
    pub fn new(id: String, name: String, format_type: FormatType) -> Self {
        Self {
            id,
            name,
            format: SerializationFormat {
                version: "1.0.0".to_string(),
                format_type,
                compression: None,
                encryption: None,
                metadata: SerializationMetadata::default(),
            },
            settings: SerializerSettings::default(),
        }
    }

    pub fn with_compression(mut self, algorithm: CompressionAlgorithm, level: u8) -> Self {
        self.format.compression = Some(CompressionInfo {
            algorithm,
            level,
            original_size: 0,
            compressed_size: 0,
        });
        self
    }

    pub fn with_encryption(mut self, algorithm: EncryptionAlgorithm, key_id: Option<String>) -> Self {
        self.format.encryption = Some(EncryptionInfo {
            algorithm,
            key_id,
            iv: None,
        });
        self
    }

    pub fn with_settings(mut self, settings: SerializerSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn serialize(&self, project: &Project) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
Create serializable project representation
        let serializable_project = self.create_serializable_project(project)?;
        
        let data = match self.format.format_type {
            FormatType::Json => self.serialize_to_json(&serializable_project)?,
            FormatType::Toml => self.serialize_to_toml(&serializable_project)?,
            FormatType::Binary => self.serialize_to_binary(&serializable_project)?,
            FormatType::Custom(ref format_name) => self.serialize_to_custom(&serializable_project, format_name)?,
        };

        let compressed_data = if let Some(compression_info) = &self.format.compression {
            self.compress_data(&data, compression_info)?
        } else {
            data
        };

        let encrypted_data = if let Some(encryption_info) = &self.format.encryption {
            self.encrypt_data(&compressed_data, encryption_info)?
        } else {
            compressed_data
        };

        Ok(encrypted_data)
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<DeserializationResult, Box<dyn std::error::Error>> {
        let (format_detected, processed_data) = self.detect_and_preprocess_data(data)?;
        
        let serializable_project = match format_detected {
            FormatType::Json => self.deserialize_from_json(&processed_data)?,
            FormatType::Toml => self.deserialize_from_toml(&processed_data)?,
            FormatType::Binary => self.deserialize_from_binary(&processed_data)?,
            FormatType::Custom(ref format_name) => self.deserialize_from_custom(&processed_data, format_name)?,
        };

        let project = self.convert_to_project(&serializable_project)?;

        let validation_result = self.validate_deserialized_project(&project);
        
        Ok(DeserializationResult {
            success: validation_result.success,
            project: if validation_result.success { Some(project) } else { None },
            warnings: validation_result.warnings,
            errors: validation_result.errors,
            metadata: DeserializationMetadata {
                format_detected,
                version_detected: "1.0.0".to_string(),
                compression_detected: self.format.compression.as_ref().map(|c| c.algorithm.clone()),
                encryption_detected: self.format.encryption.as_ref().map(|e| e.algorithm.clone()),
                schema_version: "1.0.0".to_string(),
                custom_properties: HashMap::new(),
            },
        })
    }

    fn create_serializable_project(&self, project: &Project) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        let mut serializable_assets = HashMap::new();

        if self.settings.include_assets {
            for (asset_id, asset) in &project.assets {
                let serializable_asset = self.create_serializable_asset(asset)?;
                serializable_assets.insert(asset_id.clone(), serializable_asset);
            }
        }

        Ok(SerializableProject {
            id: project.id.clone(),
            name: project.name.clone(),
            description: project.description.clone(),
            version: project.version.clone(),
            author: project.author.clone(),
            created_at: project.created_at,
            modified_at: project.modified_at,
            tags: project.tags.clone(),
            settings: self.create_serializable_settings(&project.settings)?,
            metadata: self.create_serializable_metadata(&project.metadata)?,
            assets: serializable_assets,
            state: format!("{:?}", *project.state.read()),
        })
    }

    fn create_serializable_asset(&self, asset: &Asset) -> Result<SerializableAsset, Box<dyn std::error::Error>> {
        let binary_data = if self.settings.include_binary_data && asset.size <= self.settings.max_binary_size {
            Some(self.read_asset_binary_data(&asset.path)?)
        } else {
            None
        };

        Ok(SerializableAsset {
            id: asset.id.clone(),
            name: asset.name.clone(),
            asset_type: format!("{:?}", asset.asset_type),
            path: asset.path.to_string_lossy().to_string(),
            size: asset.size,
            created_at: asset.created_at,
            modified_at: asset.modified_at,
            metadata: self.create_serializable_asset_metadata(&asset.metadata)?,
            binary_data,
        })
    }

    fn create_serializable_settings(&self, settings: &ProjectSettings) -> Result<SerializableProjectSettings, Box<dyn std::error::Error>> {
        Ok(SerializableProjectSettings {
            auto_save: settings.auto_save,
            auto_save_interval: settings.auto_save_interval.as_secs(),
            backup_enabled: settings.backup_enabled,
            backup_count: settings.backup_count,
            compression_enabled: settings.compression_enabled,
            compression_level: settings.compression_level,
            encryption_enabled: settings.encryption_enabled,
            encryption_key: settings.encryption_key.clone(),
            thumbnail_size: settings.thumbnail_size,
            preview_quality: settings.preview_quality,
            workspace_layout: format!("{:?}", settings.workspace_layout),
        })
    }

    fn create_serializable_metadata(&self, metadata: &ProjectMetadata) -> Result<SerializableProjectMetadata, Box<dyn std::error::Error>> {
        Ok(SerializableProjectMetadata {
            genre: metadata.genre.clone(),
            category: metadata.category.clone(),
            keywords: metadata.keywords.clone(),
            rating: metadata.rating,
            language: metadata.language.clone(),
            software: metadata.software.clone(),
            notes: metadata.notes.clone(),
            custom_fields: metadata.custom_fields.clone(),
        })
    }

    fn create_serializable_asset_metadata(&self, metadata: &AssetMetadata) -> Result<SerializableAssetMetadata, Box<dyn std::error::Error>> {
        Ok(SerializableAssetMetadata {
            format: metadata.format.clone(),
            dimensions: metadata.dimensions,
            duration: metadata.duration.map(|d| d.as_secs()),
            sample_rate: metadata.sample_rate,
            bit_depth: metadata.bit_depth,
            channels: metadata.channels,
            color_space: metadata.color_space.clone(),
            additional: metadata.additional.clone(),
        })
    }

    fn serialize_to_json(&self, project: &SerializableProject) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json_string = if self.settings.pretty_print {
            serde_json::to_string_pretty(project)?
        } else {
            serde_json::to_string(project)?
        };
        Ok(json_string.into_bytes())
    }

    fn serialize_to_toml(&self, project: &SerializableProject) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(project)?;
        Ok(toml_string.into_bytes())
    }

    fn serialize_to_binary(&self, project: &SerializableProject) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        bincode::serialize(project)
    }

    fn serialize_to_custom(&self, project: &SerializableProject, format_name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match format_name {
            "yaml" => {
                let yaml_string = serde_yaml::to_string(project)?;
                Ok(yaml_string.into_bytes())
            },
            "xml" => {
                let xml_string = self.serialize_to_xml(project)?;
                Ok(xml_string.into_bytes())
            },
            "csv" => {
                let csv_string = self.serialize_to_csv(project)?;
                Ok(csv_string.into_bytes())
            },
            _ => Err(format!("Unsupported custom format: {}", format_name).into()),
        }
    }

    fn serialize_to_xml(&self, project: &SerializableProject) -> Result<String, Box<dyn std::error::Error>> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project>\n");
        
        xml.push_str(&format!("  <id>{}</id>\n", project.id));
        xml.push_str(&format!("  <name>{}</name>\n", self.escape_xml(&project.name)));
        xml.push_str(&format!("  <description>{}</description>\n", self.escape_xml(&project.description)));
        xml.push_str(&format!("  <version>{}</version>\n", project.version));
        xml.push_str(&format!("  <author>{}</author>\n", self.escape_xml(&project.author)));
        xml.push_str(&format!("  <created_at>{}</created_at>\n", project.created_at.to_rfc3339()));
        xml.push_str(&format!("  <modified_at>{}</modified_at>\n", project.modified_at.to_rfc3339()));
        
        xml.push_str("  <tags>\n");
        for tag in &project.tags {
            xml.push_str(&format!("    <tag>{}</tag>\n", self.escape_xml(tag)));
        }
        xml.push_str("  </tags>\n");
        
        xml.push_str("  <assets>\n");
        for (asset_id, asset) in &project.assets {
            xml.push_str("    <asset>\n");
            xml.push_str(&format!("      <id>{}</id>\n", asset_id));
            xml.push_str(&format!("      <name>{}</name>\n", self.escape_xml(&asset.name)));
            xml.push_str(&format!("      <type>{}</type>\n", asset.asset_type));
            xml.push_str(&format!("      <path>{}</path>\n", self.escape_xml(&asset.path)));
            xml.push_str(&format!("      <size>{}</size>\n", asset.size));
            xml.push_str(&format!("      <created_at>{}</created_at>\n", asset.created_at.to_rfc3339()));
            xml.push_str(&format!("      <modified_at>{}</modified_at>\n", asset.modified_at.to_rfc3339()));
            xml.push_str("    </asset>\n");
        }
        xml.push_str("  </assets>\n");
        
        xml.push_str("</project>");
        Ok(xml)
    }

    fn serialize_to_csv(&self, project: &SerializableProject) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv = String::new();
        
        csv.push_str("id,name,description,version,author,created_at,modified_at\n");
        
        csv.push_str(&format!("{},{},{},{},{},{},{}\n",
            self.escape_csv(&project.id),
            self.escape_csv(&project.name),
            self.escape_csv(&project.description),
            self.escape_csv(&project.version),
            self.escape_csv(&project.author),
            project.created_at.to_rfc3339(),
            project.modified_at.to_rfc3339()
        ));
        
        Ok(csv)
    }

    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn escape_csv(&self, text: &str) -> String {
        if text.contains(',') || text.contains('"') || text.contains('\n') {
            format!("\"{}\"", text.replace('"', "\"\""))
        } else {
            text.to_string()
        }
    }

    fn deserialize_from_json(&self, data: &[u8]) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        let json_string = String::from_utf8(data)?;
        serde_json::from_str(&json_string)
    }

    fn deserialize_from_toml(&self, data: &[u8]) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        let toml_string = String::from_utf8(data)?;
        toml::from_str(&toml_string)
    }

    fn deserialize_from_binary(&self, data: &[u8]) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        bincode::deserialize(data)
    }

    fn deserialize_from_custom(&self, data: &[u8], format_name: &str) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        match format_name {
            "yaml" => {
                let yaml_string = String::from_utf8(data)?;
                serde_yaml::from_str(&yaml_string)
            },
            "xml" => {
                let xml_string = String::from_utf8(data)?;
                self.deserialize_from_xml(&xml_string)
            },
            "csv" => {
                let csv_string = String::from_utf8(data)?;
                self.deserialize_from_csv(&csv_string)
            },
            _ => Err(format!("Unsupported custom format: {}", format_name).into()),
        }
    }

    fn deserialize_from_xml(&self, xml_string: &str) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        let mut project = SerializableProject::default();
        
        Err("XML deserialization not fully implemented".into())
    }

    fn deserialize_from_csv(&self, csv_string: &str) -> Result<SerializableProject, Box<dyn std::error::Error>> {
        let mut project = SerializableProject::default();
        
        Err("CSV deserialization not fully implemented".into())
    }

    fn compress_data(&self, data: &[u8], compression_info: &CompressionInfo) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let original_size = data.len();
        
        let compressed = match compression_info.algorithm {
            CompressionAlgorithm::Gzip => {
                let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(compression_info.level));
                let mut encoder = encoder;
                encoder.write_all(data)?;
                encoder.finish()?
            },
            CompressionAlgorithm::Zstd => {
                Err("Zstd compression not implemented".into())
            },
            CompressionAlgorithm::Lz4 => {
                Err("LZ4 compression not implemented".into())
            },
            CompressionAlgorithm::Custom(_) => {
                Err("Custom compression not implemented".into())
            },
            CompressionAlgorithm::None => data.to_vec(),
        };

        Ok(compressed)
    }

    fn encrypt_data(&self, data: &[u8], encryption_info: &EncryptionInfo) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match encryption_info.algorithm {
            EncryptionAlgorithm::Aes256 => {
                Err("AES-256 encryption not implemented".into())
            },
            EncryptionAlgorithm::ChaCha20 => {
                Err("ChaCha20 encryption not implemented".into())
            },
            EncryptionAlgorithm::Custom(_) => {
                Err("Custom encryption not implemented".into())
            },
            EncryptionAlgorithm::None => data.to_vec(),
        }
    }

    fn detect_and_preprocess_data(&self, data: &[u8]) -> Result<(FormatType, Vec<u8>), Box<dyn std::error::Error>> {
        let format_detected = self.detect_format(data)?;
        
        let decompressed_data = if let Some(compression_info) = &self.format.compression {
            self.decompress_data(data, compression_info)?
        } else {
            data.to_vec()
        };
        
        let processed_data = if let Some(encryption_info) = &self.format.encryption {
            self.decrypt_data(&decompressed_data, encryption_info)?
        } else {
            decompressed_data
        };

        Ok((format_detected, processed_data))
    }

    fn detect_format(&self, data: &[u8]) -> Result<FormatType, Box<dyn std::error::Error>> {
        if data.starts_with(b"{") || data.starts_with(b"[") {
            return Ok(FormatType::Json);
        }
        
        if data.starts_with(b"#") || data.contains(&b"[") && data.contains(&b"=") {
            return Ok(FormatType::Toml);
        }
        
        if data.len() >= 4 && &data[0..4] == b"\x93\x94\xcd\x01" {
            return Ok(FormatType::Binary);
        }
        
        Ok(FormatType::Json)
    }

    fn decompress_data(&self, data: &[u8], compression_info: &CompressionInfo) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match compression_info.algorithm {
            CompressionAlgorithm::Gzip => {
                let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
                std::io::read_to_end(decoder)?
            },
            _ => Err("Decompression not implemented for this algorithm".into()),
        }
    }

    fn decrypt_data(&self, data: &[u8], encryption_info: &EncryptionInfo) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match encryption_info.algorithm {
            _ => Err("Decryption not implemented".into()),
        }
    }

    fn convert_to_project(&self, serializable: &SerializableProject) -> Result<Project, Box<dyn std::error::Error>> {
        let mut assets = HashMap::new();
        
        for (asset_id, asset) in &serializable.assets {
            let converted_asset = self.convert_to_asset(asset)?;
            assets.insert(asset_id.clone(), converted_asset);
        }

        Ok(Project {
            id: serializable.id.clone(),
            name: serializable.name.clone(),
            description: serializable.description.clone(),
            created_at: serializable.created_at,
            modified_at: serializable.modified_at,
            version: serializable.version.clone(),
            author: serializable.author.clone(),
            tags: serializable.tags.clone(),
            settings: self.convert_to_settings(&serializable.settings)?,
            assets,
            metadata: self.convert_to_metadata(&serializable.metadata)?,
            state: std::sync::Arc::new(parking_lot::RwLock::new(crate::project::ProjectState::Loaded))),
        })
    }

    fn convert_to_asset(&self, asset: &SerializableAsset) -> Result<Asset, Box<dyn std::error::Error>> {
        let asset_type = match asset.asset_type.as_str() {
            "Image" => AssetType::Image,
            "Video" => AssetType::Video,
            "Audio" => AssetType::Audio,
            "Text" => AssetType::Text,
            "Project" => AssetType::Project,
            "Binary" => AssetType::Binary,
            _ => AssetType::Custom(asset.asset_type.clone()),
        };

        Ok(Asset {
            id: asset.id.clone(),
            name: asset.name.clone(),
            asset_type,
            path: std::path::PathBuf::from(&asset.path),
            size: asset.size,
            created_at: asset.created_at,
            modified_at: asset.modified_at,
            metadata: self.convert_to_asset_metadata(&asset.metadata)?,
        })
    }

    fn convert_to_settings(&self, settings: &SerializableProjectSettings) -> Result<ProjectSettings, Box<dyn std::error::Error>> {
        Ok(ProjectSettings {
            auto_save: settings.auto_save,
            auto_save_interval: std::time::Duration::from_secs(settings.auto_save_interval),
            backup_enabled: settings.backup_enabled,
            backup_count: settings.backup_count,
            compression_enabled: settings.compression_enabled,
            compression_level: settings.compression_level,
            encryption_enabled: settings.encryption_enabled,
            encryption_key: settings.encryption_key.clone(),
            thumbnail_size: settings.thumbnail_size,
            preview_quality: settings.preview_quality,
            workspace_layout: self.parse_workspace_layout(&settings.workspace_layout)?,
        })
    }

    fn convert_to_metadata(&self, metadata: &SerializableProjectMetadata) -> Result<ProjectMetadata, Box<dyn std::error::Error>> {
        Ok(ProjectMetadata {
            genre: metadata.genre.clone(),
            category: metadata.category.clone(),
            keywords: metadata.keywords.clone(),
            rating: metadata.rating,
            language: metadata.language.clone(),
            software: metadata.software.clone(),
            notes: metadata.notes.clone(),
            custom_fields: metadata.custom_fields.clone(),
        })
    }

    fn convert_to_asset_metadata(&self, metadata: &SerializableAssetMetadata) -> Result<AssetMetadata, Box<dyn std::error::Error>> {
        Ok(AssetMetadata {
            format: metadata.format.clone(),
            dimensions: metadata.dimensions,
            duration: metadata.duration.map(|d| std::time::Duration::from_secs(d)),
            sample_rate: metadata.sample_rate,
            bit_depth: metadata.bit_depth,
            channels: metadata.channels,
            color_space: metadata.color_space.clone(),
            additional: metadata.additional.clone(),
        })
    }

    fn parse_workspace_layout(&self, layout_str: &str) -> Result<crate::project::WorkspaceLayout, Box<dyn std::error::Error>> {
        Ok(crate::project::WorkspaceLayout::default())
    }

    fn read_asset_binary_data(&self, path: &std::path::Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        std::fs::read(path)
    }

    fn validate_deserialized_project(&self, project: &Project) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if project.name.is_empty() {
            errors.push("Project name cannot be empty".to_string());
        }

        if project.id.is_empty() {
            errors.push("Project ID cannot be empty".to_string());
        }

        for (asset_id, asset) in &project.assets {
            if asset.name.is_empty() {
                errors.push(format!("Asset {} has empty name", asset_id));
            }
            
            if !asset.path.exists() {
                warnings.push(format!("Asset {} path does not exist: {:?}", asset_id, asset.path));
            }
        }

        ValidationReport {
            success: errors.is_empty(),
            errors,
            warnings,
        }
    }

    pub fn create_serializer_for_format(&self, format_type: FormatType) -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            format!("Serializer for {:?}", format_type),
            format_type,
        )
    }

    pub fn get_supported_formats(&self) -> Vec<FormatType> {
        vec![
            FormatType::Json,
            FormatType::Toml,
            FormatType::Binary,
        ]
    }

    pub fn get_supported_compression_algorithms(&self) -> Vec<CompressionAlgorithm> {
        vec![
            CompressionAlgorithm::None,
            CompressionAlgorithm::Gzip,
        ]
    }

    pub fn get_supported_encryption_algorithms(&self) -> Vec<EncryptionAlgorithm> {
        vec![
            EncryptionAlgorithm::None,
        ]
    }

    pub fn estimate_serialized_size(&self, project: &Project) -> Result<u64, Box<dyn std::error::Error>> {
        let serializable_project = self.create_serializable_project(project)?;
        
        let base_size = match self.format.format_type {
            FormatType::Json => serde_json::to_string(&serializable_project)?.len(),
            FormatType::Toml => toml::to_string_pretty(&serializable_project)?.len(),
            FormatType::Binary => bincode::serialize(&serializable_project)?.len(),
            FormatType::Custom(_) => 1024,
        };

        let mut final_size = base_size as u64;

        if let Some(compression_info) = &self.format.compression {
            final_size = (final_size as f64 * 1.1) as u64;
        }

        if let Some(_) = &self.format.encryption {
            final_size = (final_size as f64 * 1.05) as u64;
        }

        Ok(final_size)
    }

    pub fn create_schema(&self) -> SerializationSchema {
        SerializationSchema {
            version: "1.0.0".to_string(),
            project_fields: self.get_project_schema_fields(),
            asset_fields: self.get_asset_schema_fields(),
            metadata_fields: self.get_metadata_schema_fields(),
            format_versions: self.get_format_versions(),
        }
    }

    fn get_project_schema_fields(&self) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "id".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Unique project identifier".to_string(),
            },
            SchemaField {
                name: "name".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Project name".to_string(),
            },
            SchemaField {
                name: "description".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project description".to_string(),
            },
            SchemaField {
                name: "version".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Project version".to_string(),
            },
            SchemaField {
                name: "author".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project author".to_string(),
            },
            SchemaField {
                name: "created_at".to_string(),
                field_type: "datetime".to_string(),
                required: true,
                description: "Project creation timestamp".to_string(),
            },
            SchemaField {
                name: "modified_at".to_string(),
                field_type: "datetime".to_string(),
                required: true,
                description: "Project modification timestamp".to_string(),
            },
            SchemaField {
                name: "tags".to_string(),
                field_type: "array".to_string(),
                required: false,
                description: "Project tags".to_string(),
            },
        ]
    }

    fn get_asset_schema_fields(&self) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "id".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Unique asset identifier".to_string(),
            },
            SchemaField {
                name: "name".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Asset name".to_string(),
            },
            SchemaField {
                name: "asset_type".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Asset type".to_string(),
            },
            SchemaField {
                name: "path".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Asset file path".to_string(),
            },
            SchemaField {
                name: "size".to_string(),
                field_type: "integer".to_string(),
                required: true,
                description: "Asset file size".to_string(),
            },
            SchemaField {
                name: "created_at".to_string(),
                field_type: "datetime".to_string(),
                required: true,
                description: "Asset creation timestamp".to_string(),
            },
            SchemaField {
                name: "modified_at".to_string(),
                field_type: "datetime".to_string(),
                required: true,
                description: "Asset modification timestamp".to_string(),
            },
            SchemaField {
                name: "metadata".to_string(),
                field_type: "object".to_string(),
                required: false,
                description: "Asset metadata".to_string(),
            },
        ]
    }

    fn get_metadata_schema_fields(&self) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "genre".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project genre".to_string(),
            },
            SchemaField {
                name: "category".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project category".to_string(),
            },
            SchemaField {
                name: "keywords".to_string(),
                field_type: "array".to_string(),
                required: false,
                description: "Project keywords".to_string(),
            },
            SchemaField {
                name: "rating".to_string(),
                field_type: "number".to_string(),
                required: false,
                description: "Project rating".to_string(),
            },
            SchemaField {
                name: "language".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project language".to_string(),
            },
            SchemaField {
                name: "software".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Software used".to_string(),
            },
            SchemaField {
                name: "notes".to_string(),
                field_type: "string".to_string(),
                required: false,
                description: "Project notes".to_string(),
            },
        ]
    }

    fn get_format_versions(&self) -> Vec<String> {
        vec![
            "1.0.0".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub settings: SerializableProjectSettings,
    pub metadata: SerializableProjectMetadata,
    pub assets: HashMap<String, SerializableAsset>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAsset {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub path: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub metadata: SerializableAssetMetadata,
    pub binary_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProjectSettings {
    pub auto_save: bool,
    pub auto_save_interval: u64,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    pub thumbnail_size: (u32, u32),
    pub preview_quality: u8,
    pub workspace_layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProjectMetadata {
    pub genre: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub rating: Option<f32>,
    pub language: String,
    pub software: String,
    pub notes: String,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAssetMetadata {
    pub format: String,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub color_space: Option<String>,
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SerializationSchema {
    pub version: String,
    pub project_fields: Vec<SchemaField>,
    pub asset_fields: Vec<SchemaField>,
    pub metadata_fields: Vec<SchemaField>,
    pub format_versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
}

impl Default for ProjectSerializer {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Serializer".to_string(),
            FormatType::Json,
        )
    }
}

impl Default for SerializerSettings {
    fn default() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
            include_assets: true,
            include_binary_data: false,
            validate_after_serialization: true,
            max_binary_size: 10 * 1024 * 1024,
            chunk_size: 8192,
        }
    }
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            format_type: FormatType::Json,
            compression: None,
            encryption: None,
            metadata: SerializationMetadata::default(),
        }
    }
}

impl Default for CompressionInfo {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::None,
            level: 6,
            original_size: 0,
            compressed_size: 0,
        }
    }
}

impl Default for EncryptionInfo {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::None,
            key_id: None,
            iv: None,
        }
    }
}

impl Default for SerializationMetadata {
    fn default() -> Self {
        Self {
            created_at: chrono::Utc::now(),
            created_by: "Tiffiny Studio".to_string(),
            software_version: "1.0.0".to_string(),
            checksum: String::new(),
            schema_version: "1.0.0".to_string(),
            custom_properties: HashMap::new(),
        }
    }
}

impl Default for DeserializationResult {
    fn default() -> Self {
        Self {
            success: false,
            project: None,
            warnings: Vec::new(),
            errors: Vec::new(),
            metadata: DeserializationMetadata::default(),
        }
    }
}

impl Default for DeserializationMetadata {
    fn default() -> Self {
        Self {
            format_detected: FormatType::Json,
            version_detected: "1.0.0".to_string(),
            compression_detected: None,
            encryption_detected: None,
            schema_version: "1.0.0".to_string(),
            custom_properties: HashMap::new(),
        }
    }
}

impl Default for SerializableProject {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Project".to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            tags: Vec::new(),
            settings: SerializableProjectSettings::default(),
            metadata: SerializableProjectMetadata::default(),
            assets: HashMap::new(),
            state: "Loaded".to_string(),
        }
    }
}

impl Default for SerializableAsset {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Asset".to_string(),
            asset_type: "Binary".to_string(),
            path: "default.bin".to_string(),
            size: 0,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            metadata: SerializableAssetMetadata::default(),
            binary_data: None,
        }
    }
}

impl Default for SerializableProjectSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval: 300,
            backup_enabled: true,
            backup_count: 5,
            compression_enabled: false,
            compression_level: 6,
            encryption_enabled: false,
            encryption_key: None,
            thumbnail_size: (256, 256),
            preview_quality: 80,
            workspace_layout: "Default".to_string(),
        }
    }
}

impl Default for SerializableProjectMetadata {
    fn default() -> Self {
        Self {
            genre: "General".to_string(),
            category: "Uncategorized".to_string(),
            keywords: Vec::new(),
            rating: None,
            language: "English".to_string(),
            software: "Tiffiny Studio".to_string(),
            notes: String::new(),
            custom_fields: HashMap::new(),
        }
    }
}

impl Default for SerializableAssetMetadata {
    fn default() -> Self {
        Self {
            format: "unknown".to_string(),
            dimensions: None,
            duration: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            color_space: None,
            additional: HashMap::new(),
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            success: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl Default for SerializationSchema {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            project_fields: Vec::new(),
            asset_fields: Vec::new(),
            metadata_fields: Vec::new(),
            format_versions: vec!["1.0.0".to_string()],
        }
    }
}

impl Default for SchemaField {
    fn default() -> Self {
        Self {
            name: "field".to_string(),
            field_type: "string".to_string(),
            required: false,
            description: "Default field".to_string(),
        }
    }
}
