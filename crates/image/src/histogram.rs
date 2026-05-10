use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Histogram {
    pub id: String,
    pub bins: Vec<u32>,
    pub channel_histograms: Arc<RwLock<Vec<ChannelHistogram>>>,
    pub cumulative: Vec<u32>,
    pub min_value: f32,
    pub max_value: f32,
    pub bin_count: usize,
    pub bin_width: f32,
    pub event_sender: mpsc::UnboundedSender<HistogramEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<HistogramEvent>>>>,
}

#[derive(Debug, Clone)]
pub struct ChannelHistogram {
    pub channel: String,
    pub bins: Vec<u32>,
    pub cumulative: Vec<u32>,
    pub min_value: f32,
    pub max_value: f32,
    pub mean: f32,
    pub median: f32,
    pub mode: f32,
    pub standard_deviation: f32,
}

#[derive(Debug, Clone)]
pub enum HistogramEvent {
    HistogramUpdated,
    ChannelUpdated(String),
    Error(String),
}

impl Histogram {
    pub fn new(bin_count: usize, min_value: f32, max_value: f32) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let bin_width = if bin_count > 0 {
            (max_value - min_value) / bin_count as f32
        } else {
            1.0
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            bins: vec![0; bin_count],
            channel_histograms: Arc::new(RwLock::new(Vec::new())),
            cumulative: vec![0; bin_count],
            min_value,
            max_value,
            bin_count,
            bin_width,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn add_sample(&self, value: f32) -> Result<(), Box<dyn std::error::Error>> {
        if self.bin_count == 0 {
            return Err("Histogram has no bins".into());
        }

        let bin_index = self.get_bin_index(value)?;
        
        let mut bins = self.bins.clone();
        bins[bin_index] += 1;
        self.bins = bins;

        let _ = self.event_sender.send(HistogramEvent::HistogramUpdated);
        
        Ok(())
    }

    pub fn add_samples(&self, values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        for &value in values {
            self.add_sample(value)?;
        }
        Ok(())
    }

    pub fn add_channel_sample(&self, channel: &str, value: f32) -> Result<(), Box<dyn std::error::Error>> {
        let bin_index = self.get_bin_index(value)?;
        
        let mut channel_histograms = self.channel_histograms.write();
        
Find or create channel histogram
        let histogram_index = channel_histograms.iter().position(|h| h.channel == channel);
        
        if let Some(index) = histogram_index {
            let mut bins = channel_histograms[index].bins.clone();
            bins[bin_index] += 1;
            channel_histograms[index].bins = bins;
        } else {
            let mut new_bins = vec![0; self.bin_count];
            new_bins[bin_index] += 1;
            
            channel_histograms.push(ChannelHistogram {
                channel: channel.to_string(),
                bins: new_bins,
                cumulative: vec![0; self.bin_count],
                min_value: self.min_value,
                max_value: self.max_value,
                mean: 0.0,
                median: 0.0,
                mode: 0.0,
                standard_deviation: 0.0,
            });
        }

        let _ = self.event_sender.send(HistogramEvent::ChannelUpdated(channel.to_string()));
        
        Ok(())
    }

    pub fn add_channel_samples(&self, channel: &str, values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        for &value in values {
            self.add_channel_sample(channel, value)?;
        }
        Ok(())
    }

    pub fn add_image_samples(&self, image: &crate::image_buffer::ImageBuffer) -> Result<(), Box<dyn std::error::Error>> {
        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(pixel) = image.get_pixel(x, y) {
                    self.add_channel_sample("red", pixel.r)?;
                    self.add_channel_sample("green", pixel.g)?;
                    self.add_channel_sample("blue", pixel.b)?;
                    self.add_channel_sample("alpha", pixel.a)?;
                    
                    let luminance = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
                    self.add_channel_sample("luminance", luminance)?;
                }
            }
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cumulative = vec![0; self.bin_count];
        let mut sum = 0u64;
        
        for (i, &count) in self.bins.iter().enumerate() {
            sum += count as u64;
            cumulative[i] = sum as u32;
        }
        self.cumulative = cumulative;

        let mut channel_histograms = self.channel_histograms.write();
        
        for histogram in channel_histograms.iter_mut() {
            let mut sum = 0u64;
            let mut sum_squared = 0u64;
            let mut total_count = 0u64;
            
            for &count in histogram.bins.iter() {
                sum += count as u64;
                sum_squared += (count * count) as u64;
                total_count += count as u64;
            }
            
            if total_count > 0 {
                histogram.mean = (sum as f32) / total_count as f32;
                histogram.standard_deviation = ((sum_squared as f32 / total_count as f32) - histogram.mean * histogram.mean).sqrt();
                
                let mut cumulative_count = 0u64;
                let median_bin = self.bin_count / 2;
                
                for (i, &count) in histogram.bins.iter().enumerate() {
                    cumulative_count += count as u64;
                    if cumulative_count >= total_count / 2 {
                        histogram.median = self.bin_value_at_index(i);
                        break;
                    }
                }
                
                let mut max_count = 0u32;
                let mut mode_bin = 0;
                
                for (i, &count) in histogram.bins.iter().enumerate() {
                    if count > max_count {
                        max_count = count;
                        mode_bin = i;
                    }
                }
                
                histogram.mode = self.bin_value_at_index(mode_bin);
            }
            
            let mut channel_cumulative = vec![0; self.bin_count];
            let mut channel_sum = 0u64;
            
            for (i, &count) in histogram.bins.iter().enumerate() {
                channel_sum += count as u64;
                channel_cumulative[i] = channel_sum as u32;
            }
            
            histogram.cumulative = channel_cumulative;
        }

        let _ = self.event_sender.send(HistogramEvent::HistogramUpdated);
        
        Ok(())
    }

    pub fn get_bin_index(&self, value: f32) -> Result<usize, Box<dyn std::error::Error>> {
        if self.bin_width == 0.0 {
            return Err("Bin width is zero".into());
        }

        let index = ((value - self.min_value) / self.bin_width) as usize;
        
        if index >= self.bin_count {
            return Err("Value out of range".into());
        }
        
        Ok(index)
    }

    pub fn get_bin_value_at_index(&self, index: usize) -> f32 {
        if index >= self.bin_count {
            return self.max_value;
        }
        
        self.min_value + (index as f32 + 0.5) * self.bin_width
    }

    pub fn get_bin_count(&self) -> usize {
        self.bin_count
    }

    pub fn get_bin_range(&self, bin_index: usize) -> (f32, f32) {
        let start_value = self.min_value + bin_index as f32 * self.bin_width;
        let end_value = start_value + self.bin_width;
        
        (start_value, end_value)
    }

    pub fn get_bin_count_at_range(&self, min_value: f32, max_value: f32) -> Result<usize, Box<dyn std::error::Error>> {
        if min_value < self.min_value || max_value > self.max_value {
            return Err("Range out of histogram bounds".into());
        }

        let start_index = ((min_value - self.min_value) / self.bin_width) as usize;
        let end_index = ((max_value - self.min_value) / self.bin_width) as usize;
        
        if start_index >= self.bin_count || end_index >= self.bin_count {
            return Err("Indices out of range".into());
        }

        Ok(end_index - start_index + 1)
    }

    pub fn get_percentile(&self, percentile: f32) -> f32 {
        if self.bins.is_empty() {
            return 0.0;
        }

        let total_samples = self.cumulative.last().copied().unwrap_or(&0) as f32;
        if total_samples == 0.0 {
            return 0.0;
        }

        let target_count = (total_samples * percentile / 100.0).round() as u32;
        
        for (i, &cumulative_count) in self.cumulative.iter().enumerate() {
            if *cumulative_count >= target_count {
                return self.get_bin_value_at_index(i);
            }
        }

        self.max_value
    }

    pub fn get_median(&self) -> f32 {
        if self.cumulative.is_empty() {
            return 0.0;
        }

        let total_samples = self.cumulative.last().copied().unwrap_or(&0) as f32;
        if total_samples == 0.0 {
            return 0.0;
        }

        let median_index = (total_samples / 2.0) as usize;
        self.get_bin_value_at_index(median_index)
    }

    pub fn get_mean(&self) -> f32 {
        if self.cumulative.is_empty() {
            return 0.0;
        }

        let total_samples = self.cumulative.last().copied().unwrap_or(&0) as f32;
        if total_samples == 0.0 {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut count = 0.0;
        
        for (i, &bin_count) in self.bins.iter().enumerate() {
            let bin_value = self.get_bin_value_at_index(i);
            sum += bin_value * bin_count as f32;
            count += bin_count as f32;
        }

        if count > 0.0 {
            sum / count
        } else {
            0.0
        }
    }

    pub fn get_standard_deviation(&self) -> f32 {
        if self.cumulative.is_empty() {
            return 0.0;
        }

        let mean = self.get_mean();
        let mut sum_squared_diff = 0.0;
        let mut count = 0.0;
        
        for (i, &bin_count) in self.bins.iter().enumerate() {
            let bin_value = self.get_bin_value_at_index(i);
            let diff = bin_value - mean;
            sum_squared_diff += diff * diff * bin_count as f32;
            count += bin_count as f32;
        }

        if count > 0.0 {
            (sum_squared_diff / count).sqrt()
        } else {
            0.0
        }
    }

    pub fn get_channel_histogram(&self, channel: &str) -> Option<ChannelHistogram> {
        let channel_histograms = self.channel_histograms.read();
        channel_histograms.iter().find(|h| h.channel == channel).cloned()
    }

    pub fn get_all_channel_histograms(&self) -> Vec<ChannelHistogram> {
        self.channel_histograms.read().clone()
    }

    pub fn get_bins(&self) -> &[u32] {
        &self.bins
    }

    pub fn get_cumulative(&self) -> &[u32] {
        &self.cumulative
    }

    pub fn get_range(&self) -> (f32, f32) {
        (self.min_value, self.max_value)
    }

    pub fn clear(&self) {
        self.bins = vec![0; self.bin_count];
        self.cumulative = vec![0; self.bin_count];
        
        let mut channel_histograms = self.channel_histograms.write();
        for histogram in channel_histograms.iter_mut() {
            histogram.bins = vec![0; self.bin_count];
            histogram.cumulative = vec![0; self.bin_count];
            histogram.mean = 0.0;
            histogram.median = 0.0;
            histogram.mode = 0.0;
            histogram.standard_deviation = 0.0;
        }

        let _ = self.event_sender.send(HistogramEvent::HistogramUpdated);
    }

    pub fn clear_channel(&self, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut channel_histograms = self.channel_histograms.write();
        
        if let Some(histogram) = channel_histograms.iter_mut().find(|h| h.channel == channel) {
            histogram.bins = vec![0; self.bin_count];
            histogram.cumulative = vec![0; self.bin_count];
            histogram.mean = 0.0;
            histogram.median = 0.0;
            histogram.mode = 0.0;
            histogram.standard_deviation = 0.0;
            
            let _ = self.event_sender.send(HistogramEvent::ChannelUpdated(channel.to_string()));
            Ok(())
        } else {
            Err(format!("Channel '{}' not found", channel).into())
        }
    }

    pub fn resize(&mut self, new_bin_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        if new_bin_count == 0 {
            return Err("Bin count cannot be zero".into());
        }

        self.bin_count = new_bin_count;
        self.bin_width = (self.max_value - self.min_value) / new_bin_count as f32;
        self.bins = vec![0; new_bin_count];
        self.cumulative = vec![0; new_bin_count];
        
        let mut channel_histograms = self.channel_histograms.write();
        for histogram in channel_histograms.iter_mut() {
            histogram.bins = vec![0; new_bin_count];
            histogram.cumulative = vec![0; new_bin_count];
        }

        let _ = self.event_sender.send(HistogramEvent::HistogramUpdated);
        
        Ok(())
    }

    pub fn set_range(&mut self, min_value: f32, max_value: f32) -> Result<(), Box<dyn std::error::Error>> {
        if min_value >= max_value {
            return Err("Min value must be less than max value".into());
        }

        self.min_value = min_value;
        self.max_value = max_value;
        self.bin_width = (max_value - min_value) / self.bin_count as f32;
        
        self.clear();

        Ok(())
    }

    pub fn merge(&self, other: &Histogram) -> Result<Histogram, Box<dyn std::error::Error>> {
        if self.bin_count != other.bin_count {
            return Err("Cannot merge histograms with different bin counts".into());
        }

        let new_min = self.min_value.min(other.min_value);
        let new_max = self.max_value.max(other.max_value);
        
        let mut merged = Self::new(self.bin_count, new_min, new_max)?;
        
        for i in 0..self.bin_count {
            merged.bins[i] = self.bins[i] + other.bins[i];
        }

        merged.calculate_statistics()?;

        Ok(merged)
    }

    pub async fn get_events(&mut self) -> Vec<HistogramEvent> {
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

    pub fn get_statistics(&self) -> HistogramStats {
        let total_samples = self.cumulative.last().copied().unwrap_or(&0) as u32;
        let mean = self.get_mean();
        let median = self.get_median();
        let standard_deviation = self.get_standard_deviation();
        
        let channel_histograms = self.channel_histograms.read();
        let channel_stats: Vec<ChannelStats> = channel_histograms.iter().map(|h| {
            ChannelStats {
                channel: h.channel.clone(),
                mean: h.mean,
                median: h.median,
                mode: h.mode,
                standard_deviation: h.standard_deviation,
                total_samples: h.cumulative.last().copied().unwrap_or(&0) as u32,
            }
        }).collect();

        HistogramStats {
            bin_count: self.bin_count,
            range: (self.min_value, self.max_value),
            total_samples,
            mean,
            median,
            mode: self.get_bin_value_at_index(
                self.bins.iter().enumerate()
                    .max_by_key(|(_, &count)| *count)
                    .map(|(index, _)| index)
                    .unwrap_or(0)
            ),
            standard_deviation,
            channel_histograms: channel_stats,
        }
    }

    pub fn equalize(&self, image: &mut crate::image_buffer::ImageBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let luminance_histogram = self.get_channel_histogram("luminance")
            .ok_or("Luminance histogram not found".into())?;

        let mut lookup_table = vec![0.0; 256];
        
        let total_pixels = image.width * image.height;
        let mut cumulative = 0u32;
        
        for i in 0..256 {
            let bin_index = self.get_bin_index(i as f32)?;
            if bin_index < luminance_histogram.bins.len() {
                cumulative += luminance_histogram.bins[bin_index];
            }
            
            let target_value = (cumulative as f32 / total_pixels as f32) * 255.0;
            lookup_table[i] = target_value;
        }

        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(mut pixel) = image.get_pixel(x, y) {
                    let luminance = (0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b) as u8;
                    let equalized_luminance = lookup_table[luminance as usize];
                    let scale = equalized_luminance / luminance.max(1) as f32;
                    
                    pixel.r = (pixel.r * scale).clamp(0.0, 255.0);
                    pixel.g = (pixel.g * scale).clamp(0.0, 255.0);
                    pixel.b = (pixel.b * scale).clamp(0.0, 255.0);
                    
                    image.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(())
    }

    pub fn stretch(&self, image: &mut crate::image_buffer::ImageBuffer, min_output: f32, max_output: f32) -> Result<(), Box<dyn std::error::Error>> {
        let luminance_histogram = self.get_channel_histogram("luminance")
            .ok_or("Luminance histogram not found".into())?;

        let min_input = self.min_value;
        let max_input = self.max_value;
        
        let mut lookup_table = vec![0.0; 256];
        
        for i in 0..256 {
            let input_value = i as f32;
            let bin_index = self.get_bin_index(input_value)?;
            
            if bin_index < luminance_histogram.bins.len() {
                let cumulative = luminance_histogram.cumulative[bin_index];
                let total_pixels = luminance_histogram.cumulative.last().copied().unwrap_or(&0) as f32;
                
                if total_pixels > 0.0 {
                    let normalized = cumulative / total_pixels;
                    let stretched = (normalized * (max_output - min_output) + min_output).clamp(min_output, max_output);
                    lookup_table[i] = stretched;
                }
            }
        }

        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(mut pixel) = image.get_pixel(x, y) {
                    let luminance = (0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b) as u8;
                    let stretched_luminance = lookup_table[luminance as usize];
                    let scale = if luminance > 0 {
                        stretched_luminance / luminance as f32
                    } else {
                        1.0
                    };
                    
                    pixel.r = (pixel.r * scale).clamp(0.0, 255.0);
                    pixel.g = (pixel.g * scale).clamp(0.0, 255.0);
                    pixel.b = (pixel.b * scale).clamp(0.0, 255.0);
                    
                    image.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(())
    }

    pub fn adaptive_threshold(&self, image: &mut crate::image_buffer::ImageBuffer, window_size: u32) -> Result<(), Box<dyn std::error::Error>> {
        let half_window = window_size / 2;
        
        for y in half_window..(image.height - half_window) {
            for x in half_window..(image.width - half_window) {
                let mut window_sum = 0.0;
                let mut window_count = 0.0;
                
                for dy in -(half_window as i32)..=(half_window as i32) {
                    for dx in -(half_window as i32)..=(half_window as i32) {
                        if let Some(pixel) = image.get_pixel(
                            (x as i32 + dx).clamp(0, image.width as i32 - 1) as u32,
                            (y as i32 + dy).clamp(0, image.height as i32 - 1) as u32,
                        ) {
                            let luminance = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
                            window_sum += luminance;
                            window_count += 1.0;
                        }
                    }
                }
                
                let window_mean = if window_count > 0.0 {
                    window_sum / window_count
                } else {
                    0.0
                };
                
                if let Some(mut pixel) = image.get_pixel(x, y) {
                    let luminance = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
                    let threshold = window_mean;
                    
                    let binary_value = if luminance > threshold { 255.0 } else { 0.0 };
                    
                    pixel.r = binary_value;
                    pixel.g = binary_value;
                    pixel.b = binary_value;
                    
                    image.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(())
    }

    pub fn otsu_threshold(&self) -> f32 {
        if self.bins.is_empty() {
            return 0.0;
        }

        let total_pixels = self.cumulative.last().copied().unwrap_or(&0) as u32;
        if total_pixels == 0 {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut sum_squared = 0.0;
        let mut weight_sum = 0.0;
        let mut weight_squared_sum = 0.0;
        
        for (i, &count) in self.bins.iter().enumerate() {
            let bin_value = self.get_bin_value_at_index(i);
            let weight = count as f32;
            
            sum += bin_value * weight;
            sum_squared += bin_value * bin_value * weight;
            weight_sum += weight;
            weight_squared_sum += weight * weight;
        }

        if weight_sum > 0.0 {
            let mean = sum / weight_sum;
            let variance = (sum_squared / weight_sum) - (mean * mean);
            
            let mut w0 = 0.0;
            let mut w1 = 0.0;
            let mut u0 = 0.0;
            let mut u1 = 0.0;
            
            for (i, &count) in self.bins.iter().enumerate() {
                let bin_value = self.get_bin_value_at_index(i);
                let weight = count as f32;
                
                if i == 0 {
                    w0 = weight;
                    u0 = bin_value;
                } else {
                    w1 += weight;
                    u1 += bin_value * weight;
                }
            }
            
            let threshold = (u0 + u1) / (w0 + w1);
            
            let mut variance0 = 0.0;
            let mut variance1 = 0.0;
            
            for (i, &count) in self.bins.iter().enumerate() {
                let bin_value = self.get_bin_value_at_index(i);
                let weight = count as f32;
                
                let diff = if i == 0 {
                    bin_value - u0
                } else {
                    bin_value - threshold
                };
                
                if i == 0 {
                    variance0 += weight * diff * diff;
                } else {
                    variance1 += weight * diff * diff;
                }
            }
            
            variance0 /= w0;
            variance1 /= w1;
            
            let within_variance = (w0 * variance0 + w1 * variance1) / (w0 + w1);
            
            let threshold_value = (variance - within_variance).sqrt();
            
            threshold_value
        } else {
            0.0
        }
    }

    pub fn clone_histogram(&self) -> Histogram {
        let mut new_histogram = Self::new(self.bin_count, self.min_value, self.max_value);
        new_histogram.bins = self.bins.clone();
        new_histogram.cumulative = self.cumulative.clone();
        
        let channel_histograms = self.channel_histograms.read();
        new_histogram.channel_histograms = Arc::new(RwLock::new(channel_histograms.clone()));
        
        new_histogram
    }

    pub fn export_data(&self) -> HistogramData {
        let channel_histograms = self.channel_histograms.read();
        
        HistogramData {
            bins: self.bins.clone(),
            cumulative: self.cumulative.clone(),
            range: (self.min_value, self.max_value),
            bin_count: self.bin_count,
            channel_histograms: channel_histograms.iter().map(|h| {
                ChannelData {
                    channel: h.channel.clone(),
                    bins: h.bins.clone(),
                    cumulative: h.cumulative.clone(),
                    statistics: ChannelStatistics {
                        mean: h.mean,
                        median: h.median,
                        mode: h.mode,
                        standard_deviation: h.standard_deviation,
                    },
                }
            }).collect(),
        }
    }

    pub fn import_data(&mut self, data: HistogramData) -> Result<(), Box<dyn std::error::Error>> {
        if data.bins.len() != self.bin_count {
            return Err("Bin count mismatch".into());
        }

        self.bins = data.bins;
        self.cumulative = data.cumulative;
        self.min_value = data.range.0;
        self.max_value = data.range.1;

        let mut channel_histograms = self.channel_histograms.write();
        channel_histograms.clear();

        for channel_data in data.channel_histograms {
            if channel_data.bins.len() == self.bin_count {
                channel_histograms.push(ChannelHistogram {
                    channel: channel_data.channel.clone(),
                    bins: channel_data.bins.clone(),
                    cumulative: channel_data.cumulative.clone(),
                    min_value: self.min_value,
                    max_value: self.max_value,
                    mean: channel_data.statistics.mean,
                    median: channel_data.statistics.median,
                    mode: channel_data.statistics.mode,
                    standard_deviation: channel_data.statistics.standard_deviation,
                });
            }
        }

        let _ = self.event_sender.send(HistogramEvent::HistogramUpdated);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub bin_count: usize,
    pub range: (f32, f32),
    pub total_samples: u32,
    pub mean: f32,
    pub median: f32,
    pub mode: f32,
    pub standard_deviation: f32,
    pub channel_histograms: Vec<ChannelStats>,
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub channel: String,
    pub mean: f32,
    pub median: f32,
    pub mode: f32,
    pub standard_deviation: f32,
    pub total_samples: u32,
}

#[derive(Debug, Clone)]
pub struct HistogramData {
    pub bins: Vec<u32>,
    pub cumulative: Vec<u32>,
    pub range: (f32, f32),
    pub bin_count: usize,
    pub channel_histograms: Vec<ChannelData>,
}

#[derive(Debug, Clone)]
pub struct ChannelData {
    pub channel: String,
    pub bins: Vec<u32>,
    pub cumulative: Vec<u32>,
    pub statistics: ChannelStatistics,
}

#[derive(Debug, Clone)]
pub struct ChannelStatistics {
    pub mean: f32,
    pub median: f32,
    pub mode: f32,
    pub standard_deviation: f32,
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new(256, 0.0, 255.0)
    }
}

impl Default for ChannelHistogram {
    fn default() -> Self {
        Self {
            channel: "default".to_string(),
            bins: vec![0; 256],
            cumulative: vec![0; 256],
            min_value: 0.0,
            max_value: 255.0,
            mean: 0.0,
            median: 0.0,
            mode: 0.0,
            standard_deviation: 0.0,
        }
    }
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            bin_count: 256,
            range: (0.0, 255.0),
            total_samples: 0,
            mean: 0.0,
            median: 0.0,
            mode: 0.0,
            standard_deviation: 0.0,
            channel_histograms: Vec::new(),
        }
    }
}

impl Default for HistogramData {
    fn default() -> Self {
        Self {
            bins: vec![0; 256],
            cumulative: vec![0; 256],
            range: (0.0, 255.0),
            bin_count: 256,
            channel_histograms: Vec::new(),
        }
    }
}

impl Default for ChannelData {
    fn default() -> Self {
        Self {
            channel: "default".to_string(),
            bins: vec![0; 256],
            cumulative: vec![0; 256],
            statistics: ChannelStatistics::default(),
        }
    }
}

impl Default for ChannelStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            mode: 0.0,
            standard_deviation: 0.0,
        }
    }
}
