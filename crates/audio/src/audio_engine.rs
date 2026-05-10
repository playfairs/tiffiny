use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AudioEngine {
    pub id: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub device_manager: Arc<RwLock<AudioDeviceManager>>,
    pub mixer: Arc<RwLock<AudioMixer>>,
    pub processors: Arc<RwLock<Vec<Arc<AudioProcessor>>>>,
    pub streams: Arc<RwLock<std::collections::HashMap<String, Arc<AudioStream>>>>,
    pub event_sender: mpsc::UnboundedSender<AudioEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<AudioEvent>>>>,
    pub running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    DeviceConnected(String),
    DeviceDisconnected(String),
    StreamStarted(String),
    StreamStopped(String),
    ProcessorAdded(String),
    ProcessorRemoved(String),
    Error(String),
}

impl AudioEngine {
    pub fn new(sample_rate: u32, channels: u16, buffer_size: usize) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id: Uuid::new_v4().to_string(),
            sample_rate,
            channels,
            buffer_size,
            device_manager: Arc::new(RwLock::new(AudioDeviceManager::new())),
            mixer: Arc::new(RwLock::new(AudioMixer::new(channels))),
            processors: Arc::new(RwLock::new(Vec::new())),
            streams: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut device_manager = self.device_manager.write();
        device_manager.scan_devices().await?;
        
        let mut mixer = self.mixer.write();
        mixer.initialize(self.sample_rate, self.channels, self.buffer_size);
        
        Ok(())
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write();
        if *running {
            return Err("Audio engine is already running".into());
        }
        
        *running = true;
        
        let device_manager = self.device_manager.clone();
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            device_manager.monitor_devices(event_sender).await;
        });
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write();
        *running = false;
        
        let streams = self.streams.read();
        for stream in streams.values() {
            stream.stop().await?;
        }
        
        Ok(())
    }

    pub async fn add_processor(&self, processor: Arc<AudioProcessor>) -> Result<(), Box<dyn std::error::Error>> {
        let mut processors = self.processors.write();
        processors.push(processor.clone());
        
        let _ = self.event_sender.send(AudioEvent::ProcessorAdded(processor.id.clone()));
        Ok(())
    }

    pub async fn remove_processor(&self, processor_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut processors = self.processors.write();
        processors.retain(|p| p.id != processor_id);
        
        let _ = self.event_sender.send(AudioEvent::ProcessorRemoved(processor_id.to_string()));
        Ok(())
    }

    pub async fn create_stream(&self, stream_config: StreamConfig) -> Result<Arc<AudioStream>, Box<dyn std::error::Error>> {
        let stream = Arc::new(AudioStream::new(
            stream_config,
            self.sample_rate,
            self.channels,
            self.buffer_size,
        ));
        
        let mut streams = self.streams.write();
        streams.insert(stream.id.clone(), stream.clone());
        
        let _ = self.event_sender.send(AudioEvent::StreamStarted(stream.id.clone()));
        Ok(stream)
    }

    pub async fn remove_stream(&self, stream_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut streams = self.streams.write();
        if let Some(stream) = streams.remove(stream_id) {
            stream.stop().await?;
            let _ = self.event_sender.send(AudioEvent::StreamStopped(stream_id.to_string()));
        }
        Ok(())
    }

    pub fn get_stream(&self, stream_id: &str) -> Option<Arc<AudioStream>> {
        let streams = self.streams.read();
        streams.get(stream_id).cloned()
    }

    pub fn get_all_streams(&self) -> Vec<Arc<AudioStream>> {
        let streams = self.streams.read();
        streams.values().cloned().collect()
    }

    pub async fn process_audio(&self, input_buffer: &mut AudioBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let processors = self.processors.read();
        
        for processor in processors.iter() {
            processor.process(input_buffer).await?;
        }
        
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    pub async fn get_events(&mut self) -> Vec<AudioEvent> {
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

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        
        let streams = self.streams.read();
        for stream in streams.values() {
            stream.set_sample_rate(sample_rate);
        }
        
        let processors = self.processors.read();
        for processor in processors.iter() {
            processor.set_sample_rate(sample_rate);
        }
    }

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
        
        let mut mixer = self.mixer.write();
        mixer.set_channels(channels);
        
        let streams = self.streams.read();
        for stream in streams.values() {
            stream.set_channels(channels);
        }
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        
        let streams = self.streams.read();
        for stream in streams.values() {
            stream.set_buffer_size(buffer_size);
        }
        
        let processors = self.processors.read();
        for processor in processors.iter() {
            processor.set_buffer_size(buffer_size);
        }
    }

    pub fn get_device_manager(&self) -> Arc<RwLock<AudioDeviceManager>> {
        self.device_manager.clone()
    }

    pub fn get_mixer(&self) -> Arc<RwLock<AudioMixer>> {
        self.mixer.clone()
    }

    pub fn get_processors(&self) -> Vec<Arc<AudioProcessor>> {
        let processors = self.processors.read();
        processors.clone()
    }

    pub fn get_stats(&self) -> AudioEngineStats {
        let streams = self.streams.read();
        let processors = self.processors.read();
        let device_manager = self.device_manager.read();
        
        AudioEngineStats {
            sample_rate: self.sample_rate,
            channels: self.channels,
            buffer_size: self.buffer_size,
            active_streams: streams.len(),
            active_processors: processors.len(),
            available_devices: device_manager.get_device_count(),
            running: *self.running.read(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub name: String,
    pub stream_type: StreamType,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub latency: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Input,
    Output,
    Duplex,
}

#[derive(Debug, Clone)]
pub struct AudioEngineStats {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub active_streams: usize,
    pub active_processors: usize,
    pub available_devices: usize,
    pub running: bool,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(44100, 2, 512)
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            name: "Default Stream".to_string(),
            stream_type: StreamType::Output,
            input_device: None,
            output_device: None,
            latency: None,
        }
    }
}
