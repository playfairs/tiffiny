use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ShaderManager {
    pub id: String,
    pub name: String,
    pub shaders: Arc<RwLock<std::collections::HashMap<String, super::compute::ComputeShader>>>>,
}

#[derive(Debug, Clone)]
pub struct ShaderTemplate {
    pub id: String,
    pub name: String,
    pub template_type: ShaderTemplateType,
    pub source: String,
    pub parameters: std::collections::HashMap<String, ShaderParameter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderTemplateType {
    Compute,
    Vertex,
    Fragment,
    Geometry,
    Tessellation,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ShaderParameter {
    pub name: String,
    pub parameter_type: ShaderParameterType,
    pub default_value: String,
    pub description: String,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderParameterType {
    Float,
    Int,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Texture2D,
    Texture3D,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ShaderBuilder {
    pub id: String,
    pub name: String,
    pub template: ShaderTemplate,
    pub variables: Arc<RwLock<std::collections::HashMap<String, String>>>>,
}

#[derive(Debug, Clone)]
pub struct ShaderCache {
    pub id: String,
    pub name: String,
    pub cache: Arc<RwLock<std::collections::HashMap<String, CachedShader>>>>,
}

#[derive(Debug, Clone)]
pub struct CachedShader {
    pub shader: super::compute::ComputeShader,
    pub last_used: std::time::Instant,
    pub access_count: u64,
}

impl ShaderManager {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            shaders: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn add_shader(&self, shader: super::compute::ComputeShader) {
        let mut shaders = self.shaders.write();
        shaders.insert(shader.id.clone(), shader);
    }

    pub fn get_shader(&self, id: &str) -> Option<super::compute::ComputeShader> {
        let shaders = self.shaders.read();
        shaders.get(id).cloned()
    }

    pub fn remove_shader(&self, id: &str) -> bool {
        let mut shaders = self.shaders.write();
        if shaders.remove(id).is_some() {
            true
        } else {
            false
        }
    }

    pub fn list_shaders(&self) -> Vec<super::compute::ComputeShader> {
        let shaders = self.shaders.read();
        shaders.values().cloned().collect()
    }

    pub fn clear_shaders(&self) {
        let mut shaders = self.shaders.write();
        shaders.clear();
    }

    pub fn find_shaders_by_entry_point(&self, entry_point: &str) -> Vec<super::compute::ComputeShader> {
        let shaders = self.shaders.read();
        shaders.values()
            .filter(|shader| shader.entry_point == entry_point)
            .cloned()
            .collect()
    }

    pub fn get_shader_count(&self) -> usize {
        let shaders = self.shaders.read();
        shaders.len()
    }
}

impl ShaderTemplate {
    pub fn new(id: String, name: String, template_type: ShaderTemplateType, source: String) -> Self {
        Self {
            id,
            name,
            template_type,
            source,
            parameters: std::collections::HashMap::new(),
        }
    }

    pub fn add_parameter(&mut self, parameter: ShaderParameter) {
        self.parameters.insert(parameter.name.clone(), parameter);
    }

    pub fn get_parameter(&self, name: &str) -> Option<&ShaderParameter> {
        self.parameters.get(name)
    }

    pub fn remove_parameter(&mut self, name: &str) -> bool {
        self.parameters.remove(name).is_some()
    }

    pub fn generate_source(&self, values: &std::collections::HashMap<String, String>) -> String {
        let mut source = self.source.clone();
        
        for (key, value) in values {
            source = source.replace(&format!("{{{}}}", key), value);
        }
        
        source
    }

    pub fn validate_parameters(&self, values: &std::collections::HashMap<String, String>) -> Result<(), String> {
        for (key, value) in values {
            if let Some(param) = self.get_parameter(key) {
                match param.parameter_type {
                    ShaderParameterType::Float => {
                        if let Err(_) = value.parse::<f32>() {
                            return Err(format!("Parameter '{}' must be a valid float", key));
                        }
                        
                        if let Some(min_val) = param.min_value {
                            if let Ok(val) = value.parse::<f32>() {
                                if val < min_val {
                                    return Err(format!("Parameter '{}' must be at least {}", key, min_val));
                                }
                            }
                        }
                    },
                    
                    if let Some(max_val) = param.max_value {
                        if let Ok(val) = value.parse::<f32>() {
                            if val > max_val {
                                return Err(format!("Parameter '{}' must be at most {}", key, max_val));
                            }
                        }
                    }
                    },
                    ShaderParameterType::Int => {
                        if let Err(_) = value.parse::<i32>() {
                            return Err(format!("Parameter '{}' must be a valid integer", key));
                        }
                    },
                    ShaderParameterType::Bool => {
                        if value != "true" && value != "false" && value != "1" && value != "0" {
                            return Err(format!("Parameter '{}' must be a boolean", key));
                        }
                    },
                    ShaderParameterType::Vec2 | ShaderParameterType::Vec3 | ShaderParameterType::Vec4 => {
Validate vector format
                        if !value.starts_with('(') || !value.ends_with(')') {
                            return Err(format!("Parameter '{}' must be a vector", key));
                        }
                    },
                    ShaderParameterType::Mat3 | ShaderParameterType::Mat4 => {
                        if !value.starts_with('mat') {
                            return Err(format!("Parameter '{}' must be a matrix", key));
                        }
                    },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
}

impl ShaderBuilder {
    pub fn new(id: String, name: String, template: ShaderTemplate) -> Self {
        Self {
            id,
            name,
            template,
            variables: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn set_variable(&self, name: &str, value: &str) {
        let mut variables = self.variables.write();
        variables.insert(name.to_string(), value.to_string());
    }

    pub fn get_variable(&self, name: &str) -> Option<String> {
        let variables = self.variables.read();
        variables.get(name).cloned()
    }

    pub fn clear_variables(&self) {
        let mut variables = self.variables.write();
        variables.clear();
    }

    pub fn build_shader(&self) -> Result<String, String> {
        let variables = self.variables.read();
        
        for (key, _) in &self.template.parameters {
            if !variables.contains_key(key) {
                return Err(format!("Variable '{}' is not set", key));
            }
        }
        
        Ok(self.template.generate_source(&variables))
    }
}

impl ShaderCache {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn cache_shader(&self, shader: super::compute::ComputeShader) {
        let mut cache = self.cache.write();
        cache.insert(shader.id.clone(), CachedShader {
            shader,
            last_used: std::time::Instant::now(),
            access_count: 1,
        });
    }

    pub fn get_cached_shader(&self, id: &str) -> Option<super::compute::ComputeShader> {
        let mut cache = self.cache.write();
        if let Some(cached) = cache.get_mut(id) {
            cached.last_used = std::time::Instant::now();
            cached.access_count += 1;
            Some(cached.shader.clone())
        } else {
            None
        }
    }

    pub fn remove_cached_shader(&self, id: &str) -> Option<super::compute::ComputeShader> {
        let mut cache = self.cache.write();
        cache.remove(id).map(|cached| cached.shader)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_shaders = cache.len();
        let total_accesses: u64 = cache.values().map(|cached| cached.access_count).sum();
        let oldest_access = cache.values().map(|cached| cached.last_used).min();
        let newest_access = cache.values().map(|cached| cached.last_used).max();

        CacheStats {
            total_shaders,
            total_accesses,
            oldest_access,
            newest_access,
        }
    }

    pub fn cleanup_old_shaders(&self, max_age: std::time::Duration) -> usize {
        let mut cache = self.cache.write();
        let now = std::time::Instant::now();
        let initial_count = cache.len();
        
        cache.retain(|_, cached| now.duration_since(cached.last_used) <= max_age);
        
        initial_count - cache.len()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_shaders: usize,
    pub total_accesses: u64,
    pub oldest_access: Option<std::time::Instant>,
    pub newest_access: Option<std::time::Instant>,
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Shader Manager".to_string(),
        )
    }
}

impl Default for ShaderTemplate {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Template".to_string(),
            ShaderTemplateType::Compute,
            "Default compute shader template
@compute @workgroup_size(64)
void main(uint3 id : SV_DispatchThreadID) {
    uint index = id.x;
    
Default compute shader logic
Add your compute shader code here
}
".to_string(),
        )
    }
}

impl Default for ShaderParameter {
    fn default() -> Self {
        Self {
            name: "parameter".to_string(),
            parameter_type: ShaderParameterType::Float,
            default_value: "0.0".to_string(),
            description: "Default parameter".to_string(),
            min_value: None,
            max_value: None,
        }
    }
}

impl Default for ShaderBuilder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Shader Builder".to_string(),
            ShaderTemplate::default(),
        )
    }
}

impl Default for ShaderCache {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Shader Cache".to_string(),
        )
    }
}

impl Default for CachedShader {
    fn default() -> Self {
        Self {
            shader: super::compute::ComputeShader::default(),
            last_used: std::time::Instant::now(),
            access_count: 0,
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_shaders: 0,
            total_accesses: 0,
            oldest_access: None,
            newest_access: None,
        }
    }
}

impl Default for ShaderTemplateType {
    fn default() -> Self {
        ShaderTemplateType::Compute
    }
}

impl Default for ShaderParameterType {
    fn default() -> Self {
        ShaderParameterType::Float
    }
}
