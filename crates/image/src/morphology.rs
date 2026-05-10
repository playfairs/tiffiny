use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Morphology {
    pub id: String,
    pub name: String,
    pub operation_type: MorphologyType,
    pub kernel: Arc<RwLock<StructuringElement>>,
    pub event_sender: mpsc::UnboundedSender<MorphologyEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<MorphologyEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MorphologyType {
    Erosion,
    Dilation,
    Opening,
    Closing,
    Gradient,
    TopHat,
    BlackHat,
    HitOrMiss,
    Skeletonization,
    Thinning,
    Pruning,
}

#[derive(Debug, Clone)]
pub enum MorphologyEvent {
    KernelChanged,
    OperationCompleted,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct MorphologyResult {
    pub output_image: crate::image_buffer::ImageBuffer,
    pub operation: String,
    pub kernel_info: String,
    pub processing_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct StructuringElement {
    pub element_type: ElementType,
    pub size: u32,
    pub data: Vec<Vec<bool>>,
    pub center_x: u32,
    pub center_y: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Rectangle,
    Ellipse,
    Cross,
    Diamond,
    Octagon,
    Custom(Vec<Vec<bool>>),
}

#[derive(Debug, Clone)]
pub struct MorphologyParams {
    pub iterations: u32,
    pub border_mode: BorderMode,
    pub threshold: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorderMode {
    Constant,
    Replicate,
    Reflect,
    Wrap,
}

impl Morphology {
    pub fn new(id: String, name: String, operation_type: MorphologyType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            operation_type,
            kernel: Arc::new(RwLock::new(StructuringElement::default())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn apply(&self, input: &crate::image_buffer::ImageBuffer) -> Result<MorphologyResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let _ = self.event_sender.send(MorphologyEvent::OperationCompleted);

        let result = match self.operation_type {
            MorphologyType::Erosion => self.apply_erosion(input),
            MorphologyType::Dilation => self.apply_dilation(input),
            MorphologyType::Opening => self.apply_opening(input),
            MorphologyType::Closing => self.apply_closing(input),
            MorphologyType::Gradient => self.apply_gradient(input),
            MorphologyType::TopHat => self.apply_top_hat(input),
            MorphologyType::BlackHat => self.apply_black_hat(input),
            MorphologyType::HitOrMiss => self.apply_hit_or_miss(input),
            MorphologyType::Skeletonization => self.apply_skeletonization(input),
            MorphologyType::Thinning => self.apply_thinning(input),
            MorphologyType::Pruning => self.apply_pruning(input),
        };

        let processing_time = start_time.elapsed();
        
        match result {
            Ok(output) => Ok(MorphologyResult {
                output_image: output,
                operation: format!("{:?}", self.operation_type),
                kernel_info: self.get_kernel_info(),
                processing_time,
            }),
            Err(e) => {
                let error_msg = format!("Morphology operation failed: {}", e);
                let _ = self.event_sender.send(MorphologyEvent::Error(error_msg));
                Err(e)
            },
        }
    }

    fn apply_erosion(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = self.kernel.read();
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let mut eroded = true;
                
                for ky in 0..kernel.size {
                    for kx in 0..kernel.size {
                        if kernel.data[ky as usize][kx as usize] {
                            let src_x = x + kx - kernel.center_x;
                            let src_y = y + ky - kernel.center_y;
                            
                            if src_x < input.width && src_y < input.height {
                                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                                    if pixel.luma() == 0.0 {
                                        eroded = false;
                                        break;
                                    }
                                }
                            } else {
                                eroded = false;
                                break;
                            }
                        }
                    }
                    if !eroded { break; }
                }
                
                let output_value = if eroded { 255.0 } else { 0.0 };
                let output_pixel = crate::image_buffer::Pixel::gray(output_value);
                output.set_pixel(x, y, output_pixel);
            }
        }

        Ok(output)
    }

    fn apply_dilation(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = self.kernel.read();
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let mut dilated = false;
                
                for ky in 0..kernel.size {
                    for kx in 0..kernel.size {
                        if kernel.data[ky as usize][kx as usize] {
                            let src_x = x + kx - kernel.center_x;
                            let src_y = y + ky - kernel.center_y;
                            
                            if src_x < input.width && src_y < input.height {
                                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                                    if pixel.luma() > 0.0 {
                                        dilated = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if dilated { break; }
                }
                
                let output_value = if dilated { 255.0 } else { 0.0 };
                let output_pixel = crate::image_buffer::Pixel::gray(output_value);
                output.set_pixel(x, y, output_pixel);
            }
        }

        Ok(output)
    }

    fn apply_opening(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
Opening = Erosion followed by Dilation
        let eroded = self.apply_erosion(input)?;
        let opened = self.apply_dilation(&eroded)?;
        Ok(opened)
    }

    fn apply_closing(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let dilated = self.apply_dilation(input)?;
        let closed = self.apply_erosion(&dilated)?;
        Ok(closed)
    }

    fn apply_gradient(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let dilated = self.apply_dilation(input)?;
        let eroded = self.apply_erosion(input)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(dilated_pixel), Some(eroded_pixel)) = (dilated.get_pixel(x, y), eroded.get_pixel(x, y)) {
                    let gradient_value = (dilated_pixel.luma() - eroded_pixel.luma()).clamp(0.0, 255.0);
                    let gradient_pixel = crate::image_buffer::Pixel::gray(gradient_value);
                    output.set_pixel(x, y, gradient_pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_top_hat(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let opened = self.apply_opening(input)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(original_pixel), Some(opened_pixel)) = (input.get_pixel(x, y), opened.get_pixel(x, y)) {
                    let top_hat_value = (original_pixel.luma() - opened_pixel.luma()).clamp(0.0, 255.0);
                    let top_hat_pixel = crate::image_buffer::Pixel::gray(top_hat_value);
                    output.set_pixel(x, y, top_hat_pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_black_hat(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let closed = self.apply_closing(input)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(closed_pixel), Some(original_pixel)) = (closed.get_pixel(x, y), input.get_pixel(x, y)) {
                    let black_hat_value = (closed_pixel.luma() - original_pixel.luma()).clamp(0.0, 255.0);
                    let black_hat_pixel = crate::image_buffer::Pixel::gray(black_hat_value);
                    output.set_pixel(x, y, black_hat_pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_hit_or_miss(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = self.kernel.read();
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let mut hit = true;
                let mut miss = true;
                
                for ky in 0..kernel.size {
                    for kx in 0..kernel.size {
                        let src_x = x + kx - kernel.center_x;
                        let src_y = y + ky - kernel.center_y;
                        
                        if src_x < input.width && src_y < input.height {
                            if let Some(pixel) = input.get_pixel(src_x, src_y) {
                                let pixel_value = pixel.luma() > 0.0;
                                let kernel_value = kernel.data[ky as usize][kx as usize];
                                
                                if kernel_value && !pixel_value {
                                    hit = false;
                                }
                                if !kernel_value && pixel_value {
                                    miss = false;
                                }
                            }
                        } else {
                            hit = false;
                            miss = false;
                        }
                    }
                }
                
                let output_value = if hit && miss { 255.0 } else { 0.0 };
                let output_pixel = crate::image_buffer::Pixel::gray(output_value);
                output.set_pixel(x, y, output_pixel);
            }
        }

        Ok(output)
    }

    fn apply_skeletonization(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut current = input.clone();
        let mut changed = true;
        
        while changed {
            changed = false;
            let mut to_remove = Vec::new();
            
            for y in 1..(input.height - 1) {
                for x in 1..(input.width - 1) {
                    if let Some(pixel) = current.get_pixel(x, y) {
                        if pixel.luma() > 0.0 {
                            let neighbors = self.get_8_neighbors(&current, x, y);
                            let (p2, p3, p4, p5, p6, p7, p8, p9) = neighbors;
                            
                            let b = (p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9) as u32;
                            let a = self.count_transitions(&[p2, p3, p4, p5, p6, p7, p8, p9]);
                            
                            if b >= 2 && b <= 6 && a == 1 {
                                let cond1 = p2 * p4 * p6 == 0 && p4 * p6 * p8 == 0;
                                let cond2 = p2 * p4 * p8 == 0 && p2 * p6 * p8 == 0;
                                
                                if cond1 || cond2 {
                                    to_remove.push((x, y));
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
            
            for (x, y) in to_remove {
                let remove_pixel = crate::image_buffer::Pixel::gray(0.0);
                current.set_pixel(x, y, remove_pixel);
            }
        }

        Ok(current)
    }

    fn apply_thinning(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut current = input.clone();
        let mut changed = true;
        
        while changed {
            changed = false;
            
            changed |= self.thinning_subiteration(&mut current, 1);
            changed |= self.thinning_subiteration(&mut current, 2);
        }

        Ok(current)
    }

    fn thinning_subiteration(&self, image: &mut crate::image_buffer::ImageBuffer, iteration: i32) -> bool {
        let mut changed = false;
        let mut to_remove = Vec::new();
        
        for y in 1..(image.height - 1) {
            for x in 1..(image.width - 1) {
                if let Some(pixel) = image.get_pixel(x, y) {
                    if pixel.luma() > 0.0 {
                        let neighbors = self.get_8_neighbors(image, x, y);
                        let (p2, p3, p4, p5, p6, p7, p8, p9) = neighbors;
                        
                        let b = (p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9) as u32;
                        
                        if b >= 2 && b <= 6 {
                            if iteration == 1 {
                                let cond1 = p2 * p4 * p8 == 0 && p4 * p6 * p8 == 0;
                                let cond2 = p2 * p4 * p6 == 0 && p2 * p6 * p8 == 0;
                                
                                if cond1 && cond2 {
                                    to_remove.push((x, y));
                                    changed = true;
                                }
                            } else {
                                let cond1 = p2 * p6 * p8 == 0 && p2 * p4 * p6 == 0;
                                let cond2 = p4 * p6 * p8 == 0 && p2 * p4 * p8 == 0;
                                
                                if cond1 && cond2 {
                                    to_remove.push((x, y));
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        for (x, y) in to_remove {
            let remove_pixel = crate::image_buffer::Pixel::gray(0.0);
            image.set_pixel(x, y, remove_pixel);
        }
        
        changed
    }

    fn apply_pruning(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let skeleton = self.apply_skeletonization(input)?;
        let mut output = skeleton.clone();
        
        let mut changed = true;
        while changed {
            changed = false;
            
            for y in 1..(skeleton.height - 1) {
                for x in 1..(skeleton.width - 1) {
                    if let Some(pixel) = skeleton.get_pixel(x, y) {
                        if pixel.luma() > 0.0 {
                            let neighbors = self.get_8_neighbors(&skeleton, x, y);
                            let (p2, p3, p4, p5, p6, p7, p8, p9) = neighbors;
                            
                            let neighbor_count = (p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9) as u32;
                            
                            if neighbor_count == 1 {
                                let remove_pixel = crate::image_buffer::Pixel::gray(0.0);
                                output.set_pixel(x, y, remove_pixel);
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        Ok(output)
    }

    fn get_8_neighbors(&self, image: &crate::image_buffer::ImageBuffer, x: u32, y: u32) -> (bool, bool, bool, bool, bool, bool, bool, bool) {
        let p2 = image.get_pixel(x, y - 1).map_or(false, |p| p.luma() > 0.0);
        let p3 = image.get_pixel(x + 1, y - 1).map_or(false, |p| p.luma() > 0.0);
        let p4 = image.get_pixel(x + 1, y).map_or(false, |p| p.luma() > 0.0);
        let p5 = image.get_pixel(x + 1, y + 1).map_or(false, |p| p.luma() > 0.0);
        let p6 = image.get_pixel(x, y + 1).map_or(false, |p| p.luma() > 0.0);
        let p7 = image.get_pixel(x - 1, y + 1).map_or(false, |p| p.luma() > 0.0);
        let p8 = image.get_pixel(x - 1, y).map_or(false, |p| p.luma() > 0.0);
        let p9 = image.get_pixel(x - 1, y - 1).map_or(false, |p| p.luma() > 0.0);
        
        (p2, p3, p4, p5, p6, p7, p8, p9)
    }

    fn count_transitions(&self, neighbors: &[bool; 8]) -> u32 {
        let mut count = 0;
        
        for i in 0..7 {
            if !neighbors[i] && neighbors[i + 1] {
                count += 1;
            }
        }
        
        if !neighbors[7] && neighbors[0] {
            count += 1;
        }
        
        count
    }

    pub fn set_kernel(&self, kernel: StructuringElement) {
        let mut current_kernel = self.kernel.write();
        *current_kernel = kernel;
        
        let _ = self.event_sender.send(MorphologyEvent::KernelChanged);
    }

    pub fn set_rectangle_kernel(&self, size: u32) {
        let kernel = StructuringElement::rectangle(size);
        self.set_kernel(kernel);
    }

    pub fn set_ellipse_kernel(&self, size: u32) {
        let kernel = StructuringElement::ellipse(size);
        self.set_kernel(kernel);
    }

    pub fn set_cross_kernel(&self, size: u32) {
        let kernel = StructuringElement::cross(size);
        self.set_kernel(kernel);
    }

    pub fn set_diamond_kernel(&self, size: u32) {
        let kernel = StructuringElement::diamond(size);
        self.set_kernel(kernel);
    }

    pub fn set_octagon_kernel(&self, size: u32) {
        let kernel = StructuringElement::octagon(size);
        self.set_kernel(kernel);
    }

    pub fn set_custom_kernel(&self, data: Vec<Vec<bool>>) {
        let kernel = StructuringElement::custom(data);
        self.set_kernel(kernel);
    }

    pub fn get_kernel(&self) -> StructuringElement {
        self.kernel.read().clone()
    }

    fn get_kernel_info(&self) -> String {
        let kernel = self.kernel.read();
        format!("{:?} {}x{}", kernel.element_type, kernel.size, kernel.size)
    }

    pub async fn get_events(&mut self) -> Vec<MorphologyEvent> {
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

    pub fn clone_morphology(&self) -> Morphology {
        let mut new_morphology = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.operation_type.clone(),
        );
        
        let kernel = self.kernel.read();
        *new_morphology.kernel.write() = kernel.clone();
        
        new_morphology
    }
}

impl StructuringElement {
    pub fn rectangle(size: u32) -> Self {
        let mut data = vec![vec![true; size as usize]; size as usize];
        let center = size / 2;
        
        Self {
            element_type: ElementType::Rectangle,
            size,
            data,
            center_x: center,
            center_y: center,
        }
    }

    pub fn ellipse(size: u32) -> Self {
        let mut data = vec![vec![false; size as usize]; size as usize];
        let center = size as f32 / 2.0;
        let radius = center;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();
                
                data[y as usize][x as usize] = distance <= radius;
            }
        }
        
        Self {
            element_type: ElementType::Ellipse,
            size,
            data,
            center_x: center as u32,
            center_y: center as u32,
        }
    }

    pub fn cross(size: u32) -> Self {
        let mut data = vec![vec![false; size as usize]; size as usize];
        let center = size / 2;
        
        for i in 0..size {
            data[center as usize][i as usize] = true;
            data[i as usize][center as usize] = true;
        }
        
        Self {
            element_type: ElementType::Cross,
            size,
            data,
            center_x: center,
            center_y: center,
        }
    }

    pub fn diamond(size: u32) -> Self {
        let mut data = vec![vec![false; size as usize]; size as usize];
        let center = size as f32 / 2.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32 - center).abs();
                let dy = (y as f32 - center).abs();
                
                data[y as usize][x as usize] = dx + dy <= center;
            }
        }
        
        Self {
            element_type: ElementType::Diamond,
            size,
            data,
            center_x: center as u32,
            center_y: center as u32,
        }
    }

    pub fn octagon(size: u32) -> Self {
        let mut data = vec![vec![false; size as usize]; size as usize];
        let center = size as f32 / 2.0;
        let radius = center;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let angle = dy.atan2(dx) + std::f32::consts::PI;
                let octagon_radius = if (angle % (std::f32::consts::PI / 4.0)) < (std::f32::consts::PI / 8.0) {
                    radius
                } else {
                    radius * 0.7071
                };
                
                data[y as usize][x as usize] = distance <= octagon_radius;
            }
        }
        
        Self {
            element_type: ElementType::Octagon,
            size,
            data,
            center_x: center as u32,
            center_y: center as u32,
        }
    }

    pub fn custom(data: Vec<Vec<bool>>) -> Self {
        let size = data.len() as u32;
        let center = size / 2;
        
        Self {
            element_type: ElementType::Custom(data.clone()),
            size,
            data,
            center_x: center,
            center_y: center,
        }
    }

    pub fn get_data(&self) -> &Vec<Vec<bool>> {
        &self.data
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn get_center(&self) -> (u32, u32) {
        (self.center_x, self.center_y)
    }

    pub fn is_valid_position(&self, x: i32, y: i32) -> bool {
        let kernel_x = x + self.center_x as i32;
        let kernel_y = y + self.center_y as i32;
        
        kernel_x >= 0 && kernel_x < self.size as i32 &&
        kernel_y >= 0 && kernel_y < self.size as i32
    }

    pub fn get_value(&self, x: i32, y: i32) -> bool {
        let kernel_x = x + self.center_x as i32;
        let kernel_y = y + self.center_y as i32;
        
        if kernel_x >= 0 && kernel_x < self.size as i32 &&
           kernel_y >= 0 && kernel_y < self.size as i32 {
            self.data[kernel_y as usize][kernel_x as usize]
        } else {
            false
        }
    }

    pub fn flip_horizontal(&self) -> StructuringElement {
        let mut flipped_data = self.data.clone();
        
        for row in &mut flipped_data {
            row.reverse();
        }
        
        Self {
            element_type: self.element_type.clone(),
            size: self.size,
            data: flipped_data,
            center_x: self.center_x,
            center_y: self.center_y,
        }
    }

    pub fn flip_vertical(&self) -> StructuringElement {
        let mut flipped_data = self.data.clone();
        flipped_data.reverse();
        
        Self {
            element_type: self.element_type.clone(),
            size: self.size,
            data: flipped_data,
            center_x: self.center_x,
            center_y: self.center_y,
        }
    }

    pub fn rotate_90(&self) -> StructuringElement {
        let mut rotated_data = vec![vec![false; self.size as usize]; self.size as usize];
        
        for y in 0..self.size {
            for x in 0..self.size {
                rotated_data[x as usize][self.size - 1 - y as usize] = self.data[y as usize][x as usize];
            }
        }
        
        Self {
            element_type: self.element_type.clone(),
            size: self.size,
            data: rotated_data,
            center_x: self.center_y,
            center_y: self.size - 1 - self.center_x,
        }
    }

    pub fn dilate(&self, iterations: u32) -> StructuringElement {
        let mut current = self.clone();
        
        for _ in 0..iterations {
            let mut new_data = vec![vec![false; self.size as usize]; self.size as usize];
            
            for y in 0..self.size {
                for x in 0..self.size {
                    if current.data[y as usize][x as usize] {
                        for ky in 0..self.size {
                            for kx in 0..self.size {
                                if current.data[ky as usize][kx as usize] {
                                    let new_x = x + kx - current.center_x;
                                    let new_y = y + ky - current.center_y;
                                    
                                    if new_x < self.size && new_y < self.size {
                                        new_data[new_y as usize][new_x as usize] = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            current.data = new_data;
        }
        
        current
    }

    pub fn erode(&self, iterations: u32) -> StructuringElement {
        let mut current = self.clone();
        
        for _ in 0..iterations {
            let mut new_data = vec![vec![false; self.size as usize]; self.size as usize];
            
            for y in 0..self.size {
                for x in 0..self.size {
                    let mut fits = true;
                    
                    for ky in 0..self.size {
                        for kx in 0..self.size {
                            if current.data[ky as usize][kx as usize] {
                                let check_x = x + kx - current.center_x;
                                let check_y = y + ky - current.center_y;
                                
                                if check_x >= self.size || check_y >= self.size || 
                                   !current.data[check_y as usize][check_x as usize] {
                                    fits = false;
                                    break;
                                }
                            }
                        }
                        if !fits { break; }
                    }
                    
                    new_data[y as usize][x as usize] = fits;
                }
            }
            
            current.data = new_data;
        }
        
        current
    }
}

impl Default for Morphology {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Morphology".to_string(),
            MorphologyType::Erosion,
        )
    }
}

impl Default for MorphologyType {
    fn default() -> Self {
        MorphologyType::Erosion
    }
}

impl Default for StructuringElement {
    fn default() -> Self {
        Self::rectangle(3)
    }
}

impl Default for MorphologyParams {
    fn default() -> Self {
        Self {
            iterations: 1,
            border_mode: BorderMode::Constant,
            threshold: 0.5,
        }
    }
}

impl Default for BorderMode {
    fn default() -> Self {
        BorderMode::Constant
    }
}

impl Default for MorphologyResult {
    fn default() -> Self {
        let image = crate::image_buffer::ImageBuffer::default();
        Self {
            output_image: image,
            operation: "Erosion".to_string(),
            kernel_info: "3x3 Rectangle".to_string(),
            processing_time: std::time::Duration::from_millis(0),
        }
    }
}
