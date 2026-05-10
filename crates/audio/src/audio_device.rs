use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioDeviceManager {
    pub devices: Arc<RwLock<Vec<AudioDevice>>>,
    pub input_devices: Arc<RwLock<Vec<AudioDevice>>>,
    pub output_devices: Arc<RwLock<Vec<AudioDevice>>>,
    pub default_input_device: Arc<RwLock<Option<String>>>,
    pub default_output_device: Arc<RwLock<Option<String>>>,
    pub event_sender: mpsc::UnboundedSender<DeviceEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<DeviceEvent>>>>,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub driver: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub buffer_size: Option<usize>,
    pub latency: Option<u32>,
    pub is_default: bool,
    pub is_available: bool,
    pub capabilities: DeviceCapabilities,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Input,
    Output,
    Duplex,
}

#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub supports_low_latency: bool,
    pub supports_high_sample_rate: bool,
    pub supports_multi_channel: bool,
    pub supports_exclusive_mode: bool,
    pub max_channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub supported_formats: Vec<AudioFormat>,
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceConnected(AudioDevice),
    DeviceDisconnected(String),
    DefaultDeviceChanged(DeviceType, String),
    DeviceConfigurationChanged(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    F32,
    F64,
    I16,
    I24,
    I32,
}

impl AudioDeviceManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            devices: Arc::new(RwLock::new(Vec::new())),
            input_devices: Arc::new(RwLock::new(Vec::new())),
            output_devices: Arc::new(RwLock::new(Vec::new())),
            default_input_device: Arc::new(RwLock::new(None)),
            default_output_device: Arc::new(RwLock::new(None)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn scan_devices(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut devices = Vec::new();
        let mut input_devices = Vec::new();
        let mut output_devices = Vec::new();
        
        let simulated_devices = vec![
            AudioDevice {
                id: "default_output".to_string(),
                name: "Default Output".to_string(),
                device_type: DeviceType::Output,
                driver: "CoreAudio".to_string(),
                sample_rate: Some(44100),
                channels: Some(2),
                buffer_size: Some(512),
                latency: Some(10),
                is_default: true,
                is_available: true,
                capabilities: DeviceCapabilities {
                    supports_low_latency: true,
                    supports_high_sample_rate: true,
                    supports_multi_channel: true,
                    supports_exclusive_mode: true,
                    max_channels: 8,
                    min_sample_rate: 8000,
                    max_sample_rate: 192000,
                    supported_formats: vec![
                        AudioFormat::F32,
                        AudioFormat::I16,
                        AudioFormat::I24,
                        AudioFormat::I32,
                    ],
                },
            },
            AudioDevice {
                id: "default_input".to_string(),
                name: "Default Input".to_string(),
                device_type: DeviceType::Input,
                driver: "CoreAudio".to_string(),
                sample_rate: Some(44100),
                channels: Some(2),
                buffer_size: Some(512),
                latency: Some(10),
                is_default: true,
                is_available: true,
                capabilities: DeviceCapabilities {
                    supports_low_latency: true,
                    supports_high_sample_rate: true,
                    supports_multi_channel: true,
                    supports_exclusive_mode: true,
                    max_channels: 8,
                    min_sample_rate: 8000,
                    max_sample_rate: 192000,
                    supported_formats: vec![
                        AudioFormat::F32,
                        AudioFormat::I16,
                        AudioFormat::I24,
                        AudioFormat::I32,
                    ],
                },
            },
            AudioDevice {
                id: "usb_mic".to_string(),
                name: "USB Microphone".to_string(),
                device_type: DeviceType::Input,
                driver: "USB Audio".to_string(),
                sample_rate: Some(48000),
                channels: Some(1),
                buffer_size: Some(256),
                latency: Some(5),
                is_default: false,
                is_available: true,
                capabilities: DeviceCapabilities {
                    supports_low_latency: true,
                    supports_high_sample_rate: false,
                    supports_multi_channel: false,
                    supports_exclusive_mode: true,
                    max_channels: 2,
                    min_sample_rate: 44100,
                    max_sample_rate: 48000,
                    supported_formats: vec![
                        AudioFormat::F32,
                        AudioFormat::I16,
                    ],
                },
            },
        ];
        
        for device in simulated_devices {
            match device.device_type {
                DeviceType::Input => input_devices.push(device.clone()),
                DeviceType::Output => output_devices.push(device.clone()),
                DeviceType::Duplex => {
                    input_devices.push(device.clone());
                    output_devices.push(device.clone());
                },
            }
            devices.push(device);
        }
        
        let mut devices_guard = self.devices.write();
        *devices_guard = devices;
        
        let mut input_devices_guard = self.input_devices.write();
        *input_devices_guard = input_devices;
        
        let mut output_devices_guard = self.output_devices.write();
        *output_devices_guard = output_devices;
        
        let mut default_input = self.default_input_device.write();
        if let Some(input_device) = input_devices.iter().find(|d| d.is_default) {
            *default_input = Some(input_device.id.clone());
        }
        
        let mut default_output = self.default_output_device.write();
        if let Some(output_device) = output_devices.iter().find(|d| d.is_default) {
            *default_output = Some(output_device.id.clone());
        }
        
        Ok(())
    }

    pub async fn monitor_devices(&self, event_sender: mpsc::UnboundedSender<DeviceEvent>) {
        let devices = self.devices.clone();
        let input_devices = self.input_devices.clone();
        let output_devices = self.output_devices.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                let current_devices = devices.read().clone();
                let current_input_devices = input_devices.read().clone();
                let current_output_devices = output_devices.read().clone();
                
                if current_devices.len() == 0 {
                    let _ = event_sender.send(DeviceEvent::Error("No devices found".to_string()));
                }
            }
        });
    }

    pub fn get_device(&self, device_id: &str) -> Option<AudioDevice> {
        let devices = self.devices.read();
        devices.iter().find(|d| d.id == device_id).cloned()
    }

    pub fn get_input_devices(&self) -> Vec<AudioDevice> {
        self.input_devices.read().clone()
    }

    pub fn get_output_devices(&self) -> Vec<AudioDevice> {
        self.output_devices.read().clone()
    }

    pub fn get_all_devices(&self) -> Vec<AudioDevice> {
        self.devices.read().clone()
    }

    pub fn get_default_input_device(&self) -> Option<AudioDevice> {
        let default_id = self.default_input_device.read();
        if let Some(ref id) = *default_id {
            self.get_device(id)
        } else {
            self.input_devices.read().first().cloned()
        }
    }

    pub fn get_default_output_device(&self) -> Option<AudioDevice> {
        let default_id = self.default_output_device.read();
        if let Some(ref id) = *default_id {
            self.get_device(id)
        } else {
            self.output_devices.read().first().cloned()
        }
    }

    pub fn set_default_input_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let input_devices = self.input_devices.read();
        
        if input_devices.iter().any(|d| d.id == device_id) {
            let mut default_input = self.default_input_device.write();
            *default_input = Some(device_id.to_string());
            
            let _ = self.event_sender.send(DeviceEvent::DefaultDeviceChanged(
                DeviceType::Input,
                device_id.to_string()
            ));
            
            Ok(())
        } else {
            Err("Device not found in input devices".into())
        }
    }

    pub fn set_default_output_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output_devices = self.output_devices.read();
        
        if output_devices.iter().any(|d| d.id == device_id) {
            let mut default_output = self.default_output_device.write();
            *default_output = Some(device_id.to_string());
            
            let _ = self.event_sender.send(DeviceEvent::DefaultDeviceChanged(
                DeviceType::Output,
                device_id.to_string()
            ));
            
            Ok(())
        } else {
            Err("Device not found in output devices".into())
        }
    }

    pub fn get_device_count(&self) -> usize {
        self.devices.read().len()
    }

    pub fn get_input_device_count(&self) -> usize {
        self.input_devices.read().len()
    }

    pub fn get_output_device_count(&self) -> usize {
        self.output_devices.read().len()
    }

    pub fn is_device_available(&self, device_id: &str) -> bool {
        if let Some(device) = self.get_device(device_id) {
            device.is_available
        } else {
            false
        }
    }

    pub fn get_device_capabilities(&self, device_id: &str) -> Option<DeviceCapabilities> {
        if let Some(device) = self.get_device(device_id) {
            Some(device.capabilities)
        } else {
            None
        }
    }

    pub fn supports_sample_rate(&self, device_id: &str, sample_rate: u32) -> bool {
        if let Some(capabilities) = self.get_device_capabilities(device_id) {
            sample_rate >= capabilities.min_sample_rate && sample_rate <= capabilities.max_sample_rate
        } else {
            false
        }
    }

    pub fn supports_channels(&self, device_id: &str, channels: u16) -> bool {
        if let Some(capabilities) = self.get_device_capabilities(device_id) {
            channels <= capabilities.max_channels
        } else {
            false
        }
    }

    pub fn supports_format(&self, device_id: &str, format: &AudioFormat) -> bool {
        if let Some(capabilities) = self.get_device_capabilities(device_id) {
            capabilities.supported_formats.contains(format)
        } else {
            false
        }
    }

    pub fn get_optimal_sample_rate(&self, device_id: &str, preferred_rate: Option<u32>) -> Option<u32> {
        if let Some(device) = self.get_device(device_id) {
            if let Some(device_rate) = device.sample_rate {
                Some(device_rate)
            } else {
                let capabilities = &device.capabilities;
                
                if let Some(preferred) = preferred_rate {
                    if preferred >= capabilities.min_sample_rate && preferred <= capabilities.max_sample_rate {
                        return Some(preferred);
                    }
                }
                
                let common_rates = vec![44100, 48000, 88200, 96000];
                for &rate in &common_rates {
                    if rate >= capabilities.min_sample_rate && rate <= capabilities.max_sample_rate {
                        return Some(rate);
                    }
                }
                
                Some(capabilities.max_sample_rate)
            }
        } else {
            None
        }
    }

    pub fn get_optimal_channels(&self, device_id: &str, preferred_channels: Option<u16>) -> Option<u16> {
        if let Some(device) = self.get_device(device_id) {
            if let Some(device_channels) = device.channels {
                Some(device_channels)
            } else {
                let capabilities = &device.capabilities;
                
                if let Some(preferred) = preferred_channels {
                    if preferred <= capabilities.max_channels {
                        return Some(preferred);
                    }
                }
                
                if capabilities.max_channels >= 2 {
                    Some(2)
                } else {
                    Some(capabilities.max_channels)
                }
            }
        } else {
            None
        }
    }

    pub fn get_optimal_buffer_size(&self, device_id: &str, preferred_size: Option<usize>) -> Option<usize> {
        if let Some(device) = self.get_device(device_id) {
            if let Some(device_buffer_size) = device.buffer_size {
                Some(device_buffer_size)
            } else {
                let preferred = preferred_size.unwrap_or(512);
                
                if device.capabilities.supports_low_latency {
                    Some(preferred.min(256))
                } else {
                    Some(preferred)
                }
            }
        } else {
            None
        }
    }

    pub async fn get_events(&mut self) -> Vec<DeviceEvent> {
        let mut receiver = self.event_receiver.write();
        if let Some(ref mut rx) = *receiver {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
            events
        } else {
            Vec::new()
        }
    }

    pub fn refresh_devices(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.devices.write().clear();
        self.input_devices.write().clear();
        self.output_devices.write().clear();
        
        tokio::spawn(async {
            let _ = self.scan_devices().await;
        });
        
        Ok(())
    }

    pub fn get_device_info(&self, device_id: &str) -> Option<DeviceInfo> {
        if let Some(device) = self.get_device(device_id) {
            Some(DeviceInfo {
                id: device.id.clone(),
                name: device.name.clone(),
                device_type: device.device_type.clone(),
                driver: device.driver.clone(),
                sample_rate: device.sample_rate,
                channels: device.channels,
                buffer_size: device.buffer_size,
                latency: device.latency,
                is_default: device.is_default,
                is_available: device.is_available,
                capabilities: device.capabilities.clone(),
            })
        } else {
            None
        }
    }

    pub fn get_manager_stats(&self) -> DeviceManagerStats {
        let devices = self.devices.read();
        let input_devices = self.input_devices.read();
        let output_devices = self.output_devices.read();
        let default_input = self.default_input_device.read();
        let default_output = self.default_output_device.read();
        
        DeviceManagerStats {
            total_devices: devices.len(),
            input_devices: input_devices.len(),
            output_devices: output_devices.len(),
            default_input_device: default_input.clone(),
            default_output_device: default_output.clone(),
            available_devices: devices.iter().filter(|d| d.is_available).count(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub driver: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub buffer_size: Option<usize>,
    pub latency: Option<u32>,
    pub is_default: bool,
    pub is_available: bool,
    pub capabilities: DeviceCapabilities,
}

#[derive(Debug, Clone)]
pub struct DeviceManagerStats {
    pub total_devices: usize,
    pub input_devices: usize,
    pub output_devices: usize,
    pub default_input_device: Option<String>,
    pub default_output_device: Option<String>,
    pub available_devices: usize,
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AudioDevice {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Device".to_string(),
            device_type: DeviceType::Output,
            driver: "Unknown".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            buffer_size: Some(512),
            latency: Some(10),
            is_default: false,
            is_available: true,
            capabilities: DeviceCapabilities::default(),
        }
    }
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            supports_low_latency: false,
            supports_high_sample_rate: false,
            supports_multi_channel: false,
            supports_exclusive_mode: false,
            max_channels: 2,
            min_sample_rate: 44100,
            max_sample_rate: 44100,
            supported_formats: vec![AudioFormat::F32],
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Output
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::F32
    }
}
