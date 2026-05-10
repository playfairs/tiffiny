use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoPlayer {
    pub id: String,
    pub name: String,
    pub state: Arc<RwLock<PlayerState>>,
    pub current_video: Arc<RwLock<Option<Arc<crate::video_buffer::Buffer>>>>,
    pub current_frame: Arc<RwLock<usize>>,
    pub playback_rate: Arc<RwLock<f32>>,
    pub volume: Arc<RwLock<f32>>,
    pub muted: Arc<RwLock<bool>>,
    pub loop_enabled: Arc<RwLock<bool>>,
    pub event_sender: mpsc::UnboundedSender<PlayerEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PlayerEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Seeking,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    StateChanged(PlayerState),
    FrameChanged(usize),
    TimeChanged(std::time::Duration),
    VolumeChanged(f32),
    MutedChanged(bool),
    LoopChanged(bool),
    PlaybackRateChanged(f32),
    Error(String),
    EndOfStream,
}

#[derive(Debug, Clone)]
pub struct PlayerConfig {
    pub auto_play: bool,
    pub loop_playback: bool,
    pub default_volume: f32,
    pub default_playback_rate: f32,
    pub buffer_size: usize,
    pub hardware_acceleration: bool,
    pub subtitles_enabled: bool,
    pub audio_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub frames_played: usize,
    pub total_frames: usize,
    pub current_time: std::time::Duration,
    pub total_time: std::time::Duration,
    pub playback_rate: f32,
    pub volume: f32,
    pub is_muted: bool,
    pub is_looping: bool,
    pub buffer_utilization: f32,
}

impl VideoPlayer {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            state: Arc::new(RwLock::new(PlayerState::Stopped)),
            current_video: Arc::new(RwLock::new(None)),
            current_frame: Arc::new(RwLock::new(0)),
            playback_rate: Arc::new(RwLock::new(1.0)),
            volume: Arc::new(RwLock::new(1.0)),
            muted: Arc::new(RwLock::new(false)),
            loop_enabled: Arc::new(RwLock::new(false)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn load_video(&self, video: Arc<crate::video_buffer::Buffer>) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_video = self.current_video.write();
        *current_video = Some(video.clone());
        
        let mut current_frame = self.current_frame.write();
        *current_frame = 0;

        let _ = self.event_sender.send(PlayerEvent::FrameChanged(0));
        let _ = self.event_sender.send(PlayerEvent::TimeChanged(std::time::Duration::from_secs(0)));

        Ok(())
    }

    pub async fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_video = self.current_video.read();
        if current_video.is_none() {
            return Err("No video loaded".into());
        }

        let mut state = self.state.write();
        *state = PlayerState::Playing;

        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Playing));

Start playback loop
        self.playback_loop().await
    }

    pub async fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.write();
        *state = PlayerState::Paused;

        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Paused));
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.write();
        *state = PlayerState::Stopped;

        let mut current_frame = self.current_frame.write();
        *current_frame = 0;

        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Stopped));
        let _ = self.event_sender.send(PlayerEvent::FrameChanged(0));
        let _ = self.event_sender.send(PlayerEvent::TimeChanged(std::time::Duration::from_secs(0)));

        Ok(())
    }

    pub async fn seek(&self, time: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
        let current_video = self.current_video.read();
        if current_video.is_none() {
            return Err("No video loaded".into());
        }

        let video = current_video.as_ref().unwrap();
        let frame_rate = video.frame_rate;
        let target_frame = (time.as_secs_f64() * frame_rate) as usize;
        let total_frames = video.get_frame_count();

        if target_frame >= total_frames {
            return Err("Seek position beyond video length".into());
        }

        let mut state = self.state.write();
        *state = PlayerState::Seeking;

        let mut current_frame = self.current_frame.write();
        *current_frame = target_frame;

        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Seeking));
        let _ = self.event_sender.send(PlayerEvent::FrameChanged(target_frame));
        let _ = self.event_sender.send(PlayerEvent::TimeChanged(time));

        let new_state = PlayerState::Playing;
        *state = new_state;
        let _ = self.event_sender.send(PlayerEvent::StateChanged(new_state));

        Ok(())
    }

    pub async fn seek_frame(&self, frame: usize) -> Result<(), Box<dyn std::error::Error>> {
        let current_video = self.current_video.read();
        if current_video.is_none() {
            return Err("No video loaded".into());
        }

        let video = current_video.as_ref().unwrap();
        let total_frames = video.get_frame_count();

        if frame >= total_frames {
            return Err("Seek frame beyond video length".into());
        }

        let mut state = self.state.write();
        *state = PlayerState::Seeking;

        let mut current_frame = self.current_frame.write();
        *current_frame = frame;

        let time = std::time::Duration::from_secs_f64(frame as f64 / video.frame_rate);

        let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Seeking));
        let _ = self.event_sender.send(PlayerEvent::FrameChanged(frame));
        let _ = self.event_sender.send(PlayerEvent::TimeChanged(time));

        let new_state = PlayerState::Playing;
        *state = new_state;
        let _ = self.event_sender.send(PlayerEvent::StateChanged(new_state));

        Ok(())
    }

    pub fn set_volume(&self, volume: f32) {
        let mut current_volume = self.volume.write();
        *current_volume = volume.clamp(0.0, 1.0);

        let _ = self.event_sender.send(PlayerEvent::VolumeChanged(*current_volume));
    }

    pub fn set_muted(&self, muted: bool) {
        let mut current_muted = self.muted.write();
        *current_muted = muted;

        let _ = self.event_sender.send(PlayerEvent::MutedChanged(muted));
    }

    pub fn set_loop(&self, loop_enabled: bool) {
        let mut current_loop = self.loop_enabled.write();
        *current_loop = loop_enabled;

        let _ = self.event_sender.send(PlayerEvent::LoopChanged(loop_enabled));
    }

    pub fn set_playback_rate(&self, rate: f32) {
        let mut current_rate = self.playback_rate.write();
        *current_rate = rate.clamp(0.1, 4.0);

        let _ = self.event_sender.send(PlayerEvent::PlaybackRateChanged(*current_rate));
    }

    async fn playback_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let state = self.state.read().clone();
            
            match state {
                PlayerState::Playing => {
                    if let Some(video) = self.current_video.read().clone() {
                        let mut current_frame = self.current_frame.write();
                        let total_frames = video.get_frame_count();
                        let playback_rate = *self.playback_rate.read();

                        if *current_frame >= total_frames {
                            let loop_enabled = *self.loop_enabled.read();
                            
                            if loop_enabled {
                                *current_frame = 0;
                                let _ = self.event_sender.send(PlayerEvent::FrameChanged(0));
                                let _ = self.event_sender.send(PlayerEvent::TimeChanged(std::time::Duration::from_secs(0)));
                            } else {
                                let mut state = self.state.write();
                                *state = PlayerState::Stopped;
                                
                                let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Stopped));
                                let _ = self.event_sender.send(PlayerEvent::EndOfStream);
                                break;
                            }
                        }

                        let frame_duration = std::time::Duration::from_secs_f64(1.0 / (video.frame_rate * playback_rate));
                        tokio::time::sleep(frame_duration).await;

                        *current_frame += 1;
                        let time = std::time::Duration::from_secs_f64(*current_frame as f64 / video.frame_rate);
                        
                        let _ = self.event_sender.send(PlayerEvent::FrameChanged(*current_frame));
                        let _ = self.event_sender.send(PlayerEvent::TimeChanged(time));
                    } else {
                        break;
                    }
                },
                PlayerState::Paused => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                },
                PlayerState::Stopped => {
                    break;
                },
                PlayerState::Buffering => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    
                    let mut state = self.state.write();
                    *state = PlayerState::Playing;
                    let _ = self.event_sender.send(PlayerEvent::StateChanged(PlayerState::Playing));
                },
                PlayerState::Seeking => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                },
                PlayerState::Error(_) => {
                    break;
                },
            }
        }

        Ok(())
    }

    pub fn get_current_frame(&self) -> usize {
        *self.current_frame.read()
    }

    pub fn get_total_frames(&self) -> usize {
        self.current_video.read().as_ref().map_or(0, |v| v.get_frame_count())
    }

    pub fn get_current_time(&self) -> std::time::Duration {
        let current_frame = self.get_current_frame();
        let frame_rate = self.current_video.read().as_ref().map_or(30.0, |v| v.frame_rate);
        
        std::time::Duration::from_secs_f64(current_frame as f64 / frame_rate)
    }

    pub fn get_total_time(&self) -> std::time::Duration {
        self.current_video.read().as_ref().map_or(std::time::Duration::from_secs(0), |v| v.get_duration().unwrap_or(std::time::Duration::from_secs(0)))
    }

    pub fn get_state(&self) -> PlayerState {
        self.state.read().clone()
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    pub fn is_muted(&self) -> bool {
        *self.muted.read()
    }

    pub fn is_looping(&self) -> bool {
        *self.loop_enabled.read()
    }

    pub fn get_playback_rate(&self) -> f32 {
        *self.playback_rate.read()
    }

    pub fn get_stats(&self) -> PlayerStats {
        let current_frame = self.get_current_frame();
        let total_frames = self.get_total_frames();
        let current_time = self.get_current_time();
        let total_time = self.get_total_time();

        PlayerStats {
            frames_played: current_frame,
            total_frames,
            current_time,
            total_time,
            playback_rate: self.get_playback_rate(),
            volume: self.get_volume(),
            is_muted: self.is_muted(),
            is_looping: self.is_looping(),
            buffer_utilization: 0.0,
        }
    }

    pub fn get_progress(&self) -> f32 {
        let current_frame = self.get_current_frame();
        let total_frames = self.get_total_frames();
        
        if total_frames == 0 {
            0.0
        } else {
            (current_frame as f32 / total_frames as f32) * 100.0
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlayerState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.get_state(), PlayerState::Paused)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self.get_state(), PlayerState::Stopped)
    }

    pub fn is_buffering(&self) -> bool {
        matches!(self.get_state(), PlayerState::Buffering)
    }

    pub fn has_error(&self) -> bool {
        matches!(self.get_state(), PlayerState::Error(_))
    }

    pub fn get_error_message(&self) -> Option<String> {
        match self.get_state() {
            PlayerState::Error(msg) => Some(msg),
            _ => None,
        }
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

    pub fn step_forward(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_frame = self.get_current_frame();
        let total_frames = self.get_total_frames();
        
        if current_frame < total_frames - 1 {
            self.seek_frame(current_frame + 1)?;
        } else if self.is_looping() {
            self.seek_frame(0)?;
        }

        Ok(())
    }

    pub fn step_backward(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_frame = self.get_current_frame();
        
        if current_frame > 0 {
            self.seek_frame(current_frame - 1)?;
        }

        Ok(())
    }

    pub fn seek_forward(&self, seconds: f64) -> Result<(), Box<dyn std::error::Error>> {
        let current_time = self.get_current_time();
        let target_time = current_time + std::time::Duration::from_secs_f64(seconds);
        let total_time = self.get_total_time();
        
        let final_time = if target_time > total_time {
            if self.is_looping() {
                std::time::Duration::from_secs_f64(target_time.as_secs_f64() % total_time.as_secs_f64())
            } else {
                total_time
            }
        } else {
            target_time
        };

        self.seek(final_time)
    }

    pub fn seek_backward(&self, seconds: f64) -> Result<(), Box<dyn std::error::Error>> {
        let current_time = self.get_current_time();
        let target_time = current_time - std::time::Duration::from_secs_f64(seconds);
        
        let final_time = if target_time < std::time::Duration::from_secs(0) {
            if self.is_looping() {
                let total_time = self.get_total_time();
                std::time::Duration::from_secs_f64(total_time.as_secs_f64() + target_time.as_secs_f64())
            } else {
                std::time::Duration::from_secs(0)
            }
        } else {
            target_time
        };

        self.seek(final_time)
    }

    pub fn seek_to_percentage(&self, percentage: f32) -> Result<(), Box<dyn std::error::Error>> {
        let clamped_percentage = percentage.clamp(0.0, 100.0);
        let total_time = self.get_total_time();
        let target_time = std::time::Duration::from_secs_f64(total_time.as_secs_f64() * (clamped_percentage / 100.0));
        
        self.seek(target_time)
    }

    pub fn set_config(&self, config: PlayerConfig) {
        self.set_volume(config.default_volume);
        self.set_playback_rate(config.default_playback_rate);
        self.set_loop(config.loop_playback);
    }

    pub fn get_config(&self) -> PlayerConfig {
        PlayerConfig {
            auto_play: false,
            loop_playback: self.is_looping(),
            default_volume: self.get_volume(),
            default_playback_rate: self.get_playback_rate(),
            buffer_size: 1024,
            hardware_acceleration: false,
            subtitles_enabled: false,
            audio_enabled: true,
        }
    }

    pub fn clone_player(&self) -> VideoPlayer {
        let mut new_player = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
        );

        *new_player.state = self.state.read().clone();
        *new_player.current_video = self.current_video.read().clone();
        *new_player.current_frame = *self.current_frame.read();
        *new_player.playback_rate = *self.playback_rate.read();
        *new_player.volume = *self.volume.read();
        *new_player.muted = *self.muted.read();
        *new_player.loop_enabled = *self.loop_enabled.read();

        new_player
    }

    pub fn reset(&self) {
        let mut state = self.state.write();
        *state = PlayerState::Stopped;

        let mut current_frame = self.current_frame.write();
        *current_frame = 0;

        let mut current_video = self.current_video.write();
        *current_video = None;

        let mut playback_rate = self.playback_rate.write();
        *playback_rate = 1.0;

        let mut volume = self.volume.write();
        *volume = 1.0;

        let mut muted = self.muted.write();
        *muted = false;

        let mut loop_enabled = self.loop_enabled.write();
        *loop_enabled = false;
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Player".to_string(),
        )
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Stopped
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            auto_play: false,
            loop_playback: false,
            default_volume: 1.0,
            default_playback_rate: 1.0,
            buffer_size: 1024,
            hardware_acceleration: false,
            subtitles_enabled: false,
            audio_enabled: true,
        }
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            frames_played: 0,
            total_frames: 0,
            current_time: std::time::Duration::from_secs(0),
            total_time: std::time::Duration::from_secs(0),
            playback_rate: 1.0,
            volume: 1.0,
            is_muted: false,
            is_looping: false,
            buffer_utilization: 0.0,
        }
    }
}
