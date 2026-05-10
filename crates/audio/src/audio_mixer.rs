use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AudioMixer {
    pub id: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub tracks: Arc<RwLock<HashMap<String, Arc<MixerTrack>>>>,
    pub buses: Arc<RwLock<HashMap<String, Arc<MixerBus>>>>,
    pub master_bus: Arc<RwLock<MixerBus>>,
    pub output_buffer: Arc<RwLock<crate::audio_buffer::AudioBuffer>>,
}

#[derive(Debug, Clone)]
pub struct MixerTrack {
    pub id: String,
    pub name: String,
    pub input_buffer: Arc<RwLock<crate::audio_buffer::AudioBuffer>>,
    pub volume: Arc<RwLock<f32>>,
    pub pan: Arc<RwLock<f32>>,
    pub mute: Arc<RwLock<bool>>,
    pub solo: Arc<RwLock<bool>>,
    pub sends: Arc<RwLock<HashMap<String, f32>>>,
    pub output_bus: Arc<RwLock<Option<String>>>,
    pub effects: Arc<RwLock<Vec<Arc<crate::audio_effects::AudioEffect>>>>,
}

#[derive(Debug, Clone)]
pub struct MixerBus {
    pub id: String,
    pub name: String,
    pub input_buffer: Arc<RwLock<crate::audio_buffer::AudioBuffer>>,
    pub volume: Arc<RwLock<f32>>,
    pub pan: Arc<RwLock<f32>>,
    pub mute: Arc<RwLock<bool>>,
    pub sends: Arc<RwLock<HashMap<String, f32>>>,
    pub output_bus: Arc<RwLock<Option<String>>>,
    pub effects: Arc<RwLock<Vec<Arc<crate::audio_effects::AudioEffect>>>>,
    pub is_master: bool,
}

impl AudioMixer {
    pub fn new(channels: u16) -> Self {
        let master_bus = MixerBus::new(
            "master".to_string(),
            "Master".to_string(),
            channels,
            true
        );
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channels,
            sample_rate: 44100,
            buffer_size: 512,
            tracks: Arc::new(RwLock::new(HashMap::new())),
            buses: Arc::new(RwLock::new(HashMap::new())),
            master_bus: Arc::new(RwLock::new(master_bus)),
            output_buffer: Arc::new(RwLock::new(
                crate::audio_buffer::AudioBuffer::new(channels, 44100, 512, crate::audio_buffer::AudioFormat::F32)
            )),
        }
    }

    pub fn initialize(&mut self, sample_rate: u32, channels: u16, buffer_size: usize) {
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.buffer_size = buffer_size;
        
        let mut master_bus = self.master_bus.write();
        master_bus.initialize(sample_rate, channels, buffer_size);
        
        let tracks = self.tracks.read();
        for track in tracks.values() {
            track.initialize(sample_rate, channels, buffer_size);
        }
        
        let buses = self.buses.read();
        for bus in buses.values() {
            bus.initialize(sample_rate, channels, buffer_size);
        }
        
        let mut output_buffer = self.output_buffer.write();
        *output_buffer = crate::audio_buffer::AudioBuffer::new(channels, sample_rate, buffer_size, crate::audio_buffer::AudioFormat::F32);
    }

    pub async fn mix(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut master_bus = self.master_bus.write();
        master_bus.clear_input();
        
        let tracks = self.tracks.read();
        let mut soloed_tracks = Vec::new();
        
        for track in tracks.values() {
            if track.is_solo() {
                soloed_tracks.push(track.clone());
            }
        }
        
        let tracks_to_mix = if soloed_tracks.is_empty() {
            tracks.values().cloned().collect()
        } else {
            soloed_tracks
        };
        
        for track in &tracks_to_mix {
            if !track.is_mute() {
                self.process_track(track).await?;
            }
        }
        
        let buses = self.buses.read();
        for bus in buses.values() {
            if !bus.is_mute() {
                self.process_bus(bus).await?;
            }
        }
        
        self.process_master_bus().await?;
        
        Ok(())
    }

    async fn process_track(&self, track: &MixerTrack) -> Result<(), Box<dyn std::error::Error>> {
        let input_buffer = track.get_input_buffer();
        
        let effects = track.effects.read();
        let mut processed_buffer = input_buffer.clone();
        
        for effect in effects.iter() {
            effect.process(&mut processed_buffer).await?;
        }
        
        let volume = *track.volume.read();
        let pan = *track.pan.read();
        self.apply_volume_pan(&mut processed_buffer, volume, pan);
        
        let sends = track.sends.read();
        let buses = self.buses.read();
        
        for (bus_id, send_level) in sends.iter() {
            if let Some(bus) = buses.get(bus_id) {
                let send_buffer = processed_buffer.copy();
                send_buffer.apply_gain(*send_level);
                
                let mut bus_input = bus.input_buffer.write();
                bus_input.mix_with(&send_buffer, 1.0);
            }
        }
        
        let output_bus = track.output_bus.read();
        if let Some(ref bus_id) = *output_bus {
            if let Some(bus) = buses.get(bus_id) {
                let mut bus_input = bus.input_buffer.write();
                bus_input.mix_with(&processed_buffer, 1.0);
            }
        } else {
            let mut master_bus = self.master_bus.write();
            let mut master_input = master_bus.input_buffer.write();
            master_input.mix_with(&processed_buffer, 1.0);
        }
        
        Ok(())
    }

    async fn process_bus(&self, bus: &MixerBus) -> Result<(), Box<dyn std::error::Error>> {
        let input_buffer = bus.get_input_buffer();
        
        let effects = bus.effects.read();
        let mut processed_buffer = input_buffer.clone();
        
        for effect in effects.iter() {
            effect.process(&mut processed_buffer).await?;
        }
        
        let volume = *bus.volume.read();
        let pan = *bus.pan.read();
        self.apply_volume_pan(&mut processed_buffer, volume, pan);
        
        let sends = bus.sends.read();
        let buses = self.buses.read();
        
        for (bus_id, send_level) in sends.iter() {
            if let Some(target_bus) = buses.get(bus_id) {
                let send_buffer = processed_buffer.copy();
                send_buffer.apply_gain(*send_level);
                
                let mut bus_input = target_bus.input_buffer.write();
                bus_input.mix_with(&send_buffer, 1.0);
            }
        }
        
        let output_bus = bus.output_bus.read();
        if !bus.is_master {
            if let Some(ref bus_id) = *output_bus {
                if let Some(target_bus) = buses.get(bus_id) {
                    let mut bus_input = target_bus.input_buffer.write();
                    bus_input.mix_with(&processed_buffer, 1.0);
                }
            } else {
                let mut master_bus = self.master_bus.write();
                let mut master_input = master_bus.input_buffer.write();
                master_input.mix_with(&processed_buffer, 1.0);
            }
        }
        
        Ok(())
    }

    async fn process_master_bus(&self) -> Result<(), Box<dyn std::error::Error>> {
        let master_bus = self.master_bus.read();
        let input_buffer = master_bus.get_input_buffer();
        
        let effects = master_bus.effects.read();
        let mut processed_buffer = input_buffer.clone();
        
        for effect in effects.iter() {
            effect.process(&mut processed_buffer).await?;
        }
        
        let volume = *master_bus.volume.read();
        processed_buffer.apply_gain(volume);
        
        let mut output_buffer = self.output_buffer.write();
        let output_data = processed_buffer.clone_data();
        output_buffer.clear();
        
        for (i, &sample) in output_data.iter().enumerate() {
            output_buffer.set_sample(i % self.channels as u16, i / self.channels as usize, sample);
        }
        
        Ok(())
    }

    fn apply_volume_pan(&self, buffer: &mut crate::audio_buffer::AudioBuffer, volume: f32, pan: f32) {
        let data = buffer.data.read();
        let mut processed_data = data.clone();
        
        if self.channels >= 2 {
            let left_gain = ((1.0 - pan) / 2.0).sqrt();
            let right_gain = ((1.0 + pan) / 2.0).sqrt();
            
            for i in (0..data.len()).step_by(2) {
                if i + 1 < data.len() {
                    processed_data[i] = data[i] * left_gain * volume;
                    processed_data[i + 1] = data[i + 1] * right_gain * volume;
                }
            }
        } else {
            for (i, &sample) in data.iter().enumerate() {
                processed_data[i] = sample * volume;
            }
        }
        
        let mut data = buffer.data.write();
        *data = processed_data;
    }

    pub fn add_track(&self, track: MixerTrack) {
        let mut tracks = self.tracks.write();
        tracks.insert(track.id.clone(), Arc::new(track));
    }

    pub fn remove_track(&self, track_id: &str) -> Option<Arc<MixerTrack>> {
        let mut tracks = self.tracks.write();
        tracks.remove(track_id)
    }

    pub fn get_track(&self, track_id: &str) -> Option<Arc<MixerTrack>> {
        let tracks = self.tracks.read();
        tracks.get(track_id).cloned()
    }

    pub fn get_all_tracks(&self) -> Vec<Arc<MixerTrack>> {
        let tracks = self.tracks.read();
        tracks.values().cloned().collect()
    }

    pub fn add_bus(&self, bus: MixerBus) {
        let mut buses = self.buses.write();
        buses.insert(bus.id.clone(), Arc::new(bus));
    }

    pub fn remove_bus(&self, bus_id: &str) -> Option<Arc<MixerBus>> {
        let mut buses = self.buses.write();
        buses.remove(bus_id)
    }

    pub fn get_bus(&self, bus_id: &str) -> Option<Arc<MixerBus>> {
        let buses = self.buses.read();
        buses.get(bus_id).cloned()
    }

    pub fn get_all_buses(&self) -> Vec<Arc<MixerBus>> {
        let buses = self.buses.read();
        buses.values().cloned().collect()
    }

    pub fn get_master_bus(&self) -> Arc<RwLock<MixerBus>> {
        self.master_bus.clone()
    }

    pub fn get_output_buffer(&self) -> Arc<RwLock<crate::audio_buffer::AudioBuffer>> {
        self.output_buffer.clone()
    }

    pub fn clear_all(&self) {
        let tracks = self.tracks.read();
        for track in tracks.values() {
            track.clear_input();
        }
        
        let buses = self.buses.read();
        for bus in buses.values() {
            bus.clear_input();
        }
        
        let master_bus = self.master_bus.read();
        master_bus.clear_input();
        
        let output_buffer = self.output_buffer.read();
        output_buffer.clear();
    }

    pub fn get_mixer_stats(&self) -> MixerStats {
        let tracks = self.tracks.read();
        let buses = self.buses.read();
        let master_bus = self.master_bus.read();
        
        MixerStats {
            total_tracks: tracks.len(),
            active_tracks: tracks.values().filter(|t| !t.is_mute()).count(),
            soloed_tracks: tracks.values().filter(|t| t.is_solo()).count(),
            total_buses: buses.len(),
            active_buses: buses.values().filter(|b| !b.is_mute()).count(),
            master_volume: *master_bus.volume.read(),
            master_mute: *master_bus.mute.read(),
            channels: self.channels,
            sample_rate: self.sample_rate,
            buffer_size: self.buffer_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MixerStats {
    pub total_tracks: usize,
    pub active_tracks: usize,
    pub soloed_tracks: usize,
    pub total_buses: usize,
    pub active_buses: usize,
    pub master_volume: f32,
    pub master_mute: bool,
    pub channels: u16,
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl MixerTrack {
    pub fn new(id: String, name: String, channels: u16) -> Self {
        Self {
            id,
            name,
            input_buffer: Arc::new(RwLock::new(
                crate::audio_buffer::AudioBuffer::new(channels, 44100, 512, crate::audio_buffer::AudioFormat::F32)
            )),
            volume: Arc::new(RwLock::new(1.0)),
            pan: Arc::new(RwLock::new(0.0)),
            mute: Arc::new(RwLock::new(false)),
            solo: Arc::new(RwLock::new(false)),
            sends: Arc::new(RwLock::new(HashMap::new())),
            output_bus: Arc::new(RwLock::new(None)),
            effects: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(&self, sample_rate: u32, channels: u16, buffer_size: usize) {
        let mut input_buffer = self.input_buffer.write();
        *input_buffer = crate::audio_buffer::AudioBuffer::new(channels, sample_rate, buffer_size, crate::audio_buffer::AudioFormat::F32);
    }

    pub fn get_input_buffer(&self) -> crate::audio_buffer::AudioBuffer {
        self.input_buffer.read().clone()
    }

    pub fn clear_input(&self) {
        let input_buffer = self.input_buffer.read();
        input_buffer.clear();
    }

    pub fn set_volume(&self, volume: f32) {
        let mut vol = self.volume.write();
        *vol = volume.clamp(0.0, 2.0);
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    pub fn set_pan(&self, pan: f32) {
        let mut p = self.pan.write();
        *p = pan.clamp(-1.0, 1.0);
    }

    pub fn get_pan(&self) -> f32 {
        *self.pan.read()
    }

    pub fn set_mute(&self, mute: bool) {
        let mut m = self.mute.write();
        *m = mute;
    }

    pub fn is_mute(&self) -> bool {
        *self.mute.read()
    }

    pub fn set_solo(&self, solo: bool) {
        let mut s = self.solo.write();
        *s = solo;
    }

    pub fn is_solo(&self) -> bool {
        *self.solo.read()
    }

    pub fn set_send(&self, bus_id: String, level: f32) {
        let mut sends = self.sends.write();
        sends.insert(bus_id, level.clamp(0.0, 1.0));
    }

    pub fn get_send(&self, bus_id: &str) -> Option<f32> {
        let sends = self.sends.read();
        sends.get(bus_id).copied()
    }

    pub fn remove_send(&self, bus_id: &str) {
        let mut sends = self.sends.write();
        sends.remove(bus_id);
    }

    pub fn set_output_bus(&self, bus_id: Option<String>) {
        let mut output = self.output_bus.write();
        *output = bus_id;
    }

    pub fn get_output_bus(&self) -> Option<String> {
        self.output_bus.read().clone()
    }

    pub fn add_effect(&self, effect: Arc<crate::audio_effects::AudioEffect>) {
        let mut effects = self.effects.write();
        effects.push(effect);
    }

    pub fn remove_effect(&self, effect_id: &str) -> Option<Arc<crate::audio_effects::AudioEffect>> {
        let mut effects = self.effects.write();
        let index = effects.iter().position(|e| e.id == effect_id);
        if let Some(index) = index {
            Some(effects.remove(index))
        } else {
            None
        }
    }

    pub fn get_effects(&self) -> Vec<Arc<crate::audio_effects::AudioEffect>> {
        self.effects.read().clone()
    }
}

impl MixerBus {
    pub fn new(id: String, name: String, channels: u16, is_master: bool) -> Self {
        Self {
            id,
            name,
            input_buffer: Arc::new(RwLock::new(
                crate::audio_buffer::AudioBuffer::new(channels, 44100, 512, crate::audio_buffer::AudioFormat::F32)
            )),
            volume: Arc::new(RwLock::new(1.0)),
            pan: Arc::new(RwLock::new(0.0)),
            mute: Arc::new(RwLock::new(false)),
            sends: Arc::new(RwLock::new(HashMap::new())),
            output_bus: Arc::new(RwLock::new(None)),
            effects: Arc::new(RwLock::new(Vec::new())),
            is_master,
        }
    }

    pub fn initialize(&self, sample_rate: u32, channels: u16, buffer_size: usize) {
        let mut input_buffer = self.input_buffer.write();
        *input_buffer = crate::audio_buffer::AudioBuffer::new(channels, sample_rate, buffer_size, crate::audio_buffer::AudioFormat::F32);
    }

    pub fn get_input_buffer(&self) -> crate::audio_buffer::AudioBuffer {
        self.input_buffer.read().clone()
    }

    pub fn clear_input(&self) {
        let input_buffer = self.input_buffer.read();
        input_buffer.clear();
    }

    pub fn set_volume(&self, volume: f32) {
        let mut vol = self.volume.write();
        *vol = volume.clamp(0.0, 2.0);
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    pub fn set_pan(&self, pan: f32) {
        let mut p = self.pan.write();
        *p = pan.clamp(-1.0, 1.0);
    }

    pub fn get_pan(&self) -> f32 {
        *self.pan.read()
    }

    pub fn set_mute(&self, mute: bool) {
        let mut m = self.mute.write();
        *m = mute;
    }

    pub fn is_mute(&self) -> bool {
        *self.mute.read()
    }

    pub fn set_send(&self, bus_id: String, level: f32) {
        let mut sends = self.sends.write();
        sends.insert(bus_id, level.clamp(0.0, 1.0));
    }

    pub fn get_send(&self, bus_id: &str) -> Option<f32> {
        let sends = self.sends.read();
        sends.get(bus_id).copied()
    }

    pub fn remove_send(&self, bus_id: &str) {
        let mut sends = self.sends.write();
        sends.remove(bus_id);
    }

    pub fn set_output_bus(&self, bus_id: Option<String>) {
        let mut output = self.output_bus.write();
        *output = bus_id;
    }

    pub fn get_output_bus(&self) -> Option<String> {
        self.output_bus.read().clone()
    }

    pub fn add_effect(&self, effect: Arc<crate::audio_effects::AudioEffect>) {
        let mut effects = self.effects.write();
        effects.push(effect);
    }

    pub fn remove_effect(&self, effect_id: &str) -> Option<Arc<crate::audio_effects::AudioEffect>> {
        let mut effects = self.effects.write();
        let index = effects.iter().position(|e| e.id == effect_id);
        if let Some(index) = index {
            Some(effects.remove(index))
        } else {
            None
        }
    }

    pub fn get_effects(&self) -> Vec<Arc<crate::audio_effects::AudioEffect>> {
        self.effects.read().clone()
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Default for MixerTrack {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Track".to_string(),
            2
        )
    }
}

impl Default for MixerBus {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Bus".to_string(),
            2,
            false
        )
    }
}
