use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder, Source};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct AudioPlayer {
    pub id: String,
    pub state: Arc<RwLock<PlayerState>>,
    pub volume: Arc<RwLock<f32>>,
    pub position: Arc<RwLock<f64>>,
    pub duration: Arc<RwLock<f64>>,
    pub loop_mode: Arc<RwLock<LoopMode>>,
    pub current_track: Arc<RwLock<Option<Track>>>,
    pub queue: Arc<RwLock<Vec<Track>>>,
    pub event_sender: mpsc::UnboundedSender<PlayerEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PlayerEvent>>>>,
    pub stream_handle: Arc<RwLock<Option<OutputStreamHandle>>>,
    pub sink: Arc<RwLock<Option<Sink>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
    Loading,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopMode {
    None,
    Track,
    Queue,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub file_path: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    TrackChanged(Track),
    StateChanged(PlayerState),
    PositionChanged(f64),
    VolumeChanged(f32),
    LoopModeChanged(LoopMode),
    QueueChanged(Vec<Track>),
    Error(String),
}

impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let (stream, stream_handle) = OutputStream::new()?;
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: Arc::new(RwLock::new(PlayerState::Stopped)),
            volume: Arc::new(RwLock::new(1.0)),
            position: Arc::new(RwLock::new(0.0)),
            duration: Arc::new(RwLock::new(0.0)),
            loop_mode: Arc::new(RwLock::new(LoopMode::None)),
            current_track: Arc::new(RwLock::new(None)),
            queue: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            stream_handle: Arc::new(RwLock::new(Some(stream_handle))),
            sink: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn load_track(&self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.write();
        *state = PlayerState::Loading;
        
        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Loading));
        
        let audio_data = std::fs::read(&track.file_path)?;
        let cursor = Cursor::new(audio_data);
        let decoder = Decoder::new(cursor)?;
        
        let stream_handle = self.stream_handle.read();
        if let Some(ref handle) = *stream_handle {
            let sink = Sink::try_new(handle)?;
            
            sink.append(decoder);
            
            let mut sink_guard = self.sink.write();
            *sink_guard = Some(sink);
        }
        
        let mut current_track = self.current_track.write();
        *current_track = Some(track.clone());
        
        let mut duration = self.duration.write();
        *duration = track.duration;
        
        let mut position = self.position.write();
        *position = 0.0;
        
        let mut state = self.state.write();
        *state = PlayerState::Stopped;
        
        let _ = self.event_sender.send(PlayerEvent::TrackChanged(track));
        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Stopped));
        
        Ok(())
    }

    pub async fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sink = self.sink.read();
        
        if let Some(ref sink) = *sink {
            sink.play();
            
            let mut state = self.state.write();
            *state = PlayerState::Playing;
            
            let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Playing));
            
            self.start_position_tracking().await;
        }
        
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sink = self.sink.read();
        
        if let Some(ref sink) = *sink {
            sink.pause();
            
            let mut state = self.state.write();
            *state = PlayerState::Paused;
            
            let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Paused));
        }
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sink = self.sink.read();
        
        if let Some(ref sink) = *sink {
            sink.stop();
            
            let mut state = self.state.write();
            *state = PlayerState::Stopped;
            
            let mut position = self.position.write();
            *position = 0.0;
            
            let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Stopped));
            let _ = self.event_sender.send(PlayerEvent::PositionChanged(0.0));
        }
        
        Ok(())
    }

    pub async fn seek(&self, position: f64) -> Result<(), Box<dyn std::error::Error>> {
        let current_track = self.current_track.read();
        
        if let Some(ref track) = *current_track {
            let mut position_guard = self.position.write();
            *position_guard = position.clamp(0.0, track.duration);
            
            let _ = self.event_sender.send(PlayerEvent::PositionChanged(position));
        }
        
        Ok(())
    }

    pub async fn next_track(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.queue.write();
        
        if let Some(track) = queue.first() {
            let track = track.clone();
            queue.remove(0);
            
            drop(queue);
            
            self.load_track(track).await?;
            self.play().await?;
            
            let _ = self.event_sender.send(PlayerEvent::QueueChanged(queue.clone()));
        }
        
        Ok(())
    }

    pub async fn previous_track(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.seek(0.0).await?;
        self.play().await?;
        
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        let mut volume_guard = self.volume.write();
        *volume_guard = clamped_volume;
        
        let sink = self.sink.read();
        if let Some(ref sink) = *sink {
            sink.set_volume(clamped_volume);
        }
        
        let _ = self.event_sender.send(PlayerEvent::VolumeChanged(clamped_volume));
        
        Ok(())
    }

    pub fn set_loop_mode(&self, loop_mode: LoopMode) -> Result<(), Box<dyn std::error::Error>> {
        let mut loop_mode_guard = self.loop_mode.write();
        *loop_mode_guard = loop_mode.clone();
        
        let _ = self.event_sender.send(PlayerEvent::LoopModeChanged(loop_mode));
        
        Ok(())
    }

    pub fn add_to_queue(&self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.queue.write();
        queue.push(track.clone());
        
        let _ = self.event_sender.send(PlayerEvent::QueueChanged(queue.clone()));
        
        Ok(())
    }

    pub fn remove_from_queue(&self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.queue.write();
        
        if index < queue.len() {
            queue.remove(index);
            
            let _ = self.event_sender.send(PlayerEvent::QueueChanged(queue.clone()));
        }
        
        Ok(())
    }

    pub fn clear_queue(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.queue.write();
        queue.clear();
        
        let _ = self.event_sender.send(PlayerEvent::QueueChanged(queue.clone()));
        
        Ok(())
    }

    pub fn shuffle_queue(&self) -> Result<(), Box<dyn std::error::Error>> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        let mut queue = self.queue.write();
        queue.shuffle(&mut thread_rng());
        
        let _ = self.event_sender.send(PlayerEvent::QueueChanged(queue.clone()));
        
        Ok(())
    }

    pub fn get_state(&self) -> PlayerState {
        self.state.read().clone()
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    pub fn get_position(&self) -> f64 {
        *self.position.read()
    }

    pub fn get_duration(&self) -> f64 {
        *self.duration.read()
    }

    pub fn get_current_track(&self) -> Option<Track> {
        self.current_track.read().clone()
    }

    pub fn get_queue(&self) -> Vec<Track> {
        self.queue.read().clone()
    }

    pub fn get_loop_mode(&self) -> LoopMode {
        self.loop_mode.read().clone()
    }

    pub fn is_playing(&self) -> bool {
        matches!(*self.state.read(), PlayerState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(*self.state.read(), PlayerState::Paused)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(*self.state.read(), PlayerState::Stopped)
    }

    async fn start_position_tracking(&self) {
        let position = self.position.clone();
        let duration = self.duration.clone();
        let state = self.state.clone();
        let loop_mode = self.loop_mode.clone();
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                let current_state = state.read().clone();
                if !matches!(current_state, PlayerState::Playing) {
                    break;
                }
                
                let mut pos = position.write();
                *pos += 0.1;
                
                let current_duration = *duration.read();
                let current_loop_mode = loop_mode.read().clone();
                
                if *pos >= current_duration {
                    match current_loop_mode {
                        LoopMode::Track => {
                            *pos = 0.0;
                        },
                        LoopMode::Queue => {
                            break;
                        },
                        LoopMode::None => {
                            *pos = current_duration;
                            let mut state_guard = state.write();
                            *state_guard = PlayerState::Stopped;
                            break;
                        },
                    }
                }
                
                let _ = event_sender.send(PlayerEvent::PositionChanged(*pos));
            }
        });
    }

    pub async fn get_events(&mut self) -> Vec<PlayerEvent> {
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

    pub fn get_stats(&self) -> PlayerStats {
        let current_track = self.current_track.read();
        let queue = self.queue.read();
        
        PlayerStats {
            state: self.state.read().clone(),
            volume: *self.volume.read(),
            position: *self.position.read(),
            duration: *self.duration.read(),
            loop_mode: self.loop_mode.read().clone(),
            current_track: current_track.clone(),
            queue_length: queue.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub state: PlayerState,
    pub volume: f32,
    pub position: f64,
    pub duration: f64,
    pub loop_mode: LoopMode,
    pub current_track: Option<Track>,
    pub queue_length: usize,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create audio player")
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Unknown Track".to_string(),
            artist: None,
            album: None,
            duration: 0.0,
            file_path: String::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for LoopMode {
    fn default() -> Self {
        LoopMode::None
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Stopped
    }
}
