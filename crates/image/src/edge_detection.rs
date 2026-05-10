use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct EdgeDetector {
    pub id: String,
    pub name: String,
    pub detector_type: EdgeDetectorType,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<EdgeDetectionEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<EdgeDetectionEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDetectorType {
    Sobel,
    Prewitt,
    Roberts,
    Laplacian,
    Canny,
    Scharr,
    FreiChen,
    Kirsch,
    Robinson,
    NevatiaBabu,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum EdgeDetectionEvent {
    ParameterChanged(String, f32),
    DetectionCompleted,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct EdgeDetectionResult {
    pub edge_image: crate::image_buffer::ImageBuffer,
    pub method: String,
    pub parameters: std::collections::HashMap<String, f32>,
    pub edge_count: u32,
    pub edge_strength: f32,
}

#[derive(Debug, Clone)]
pub struct SobelParams {
    pub kernel_size: u32,
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct PrewittParams {
    pub kernel_size: u32,
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct RobertsParams {
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct LaplacianParams {
    pub kernel_size: u32,
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct CannyParams {
    pub low_threshold: f32,
    pub high_threshold: f32,
    pub kernel_size: u32,
    pub sigma: f32,
}

#[derive(Debug, Clone)]
pub struct ScharrParams {
    pub kernel_size: u32,
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct FreiChenParams {
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct KirschParams {
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct RobinsonParams {
    pub threshold: f32,
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct NevatiaBabuParams {
    pub threshold: f32,
    pub normalize: bool,
}

impl EdgeDetector {
    pub fn new(id: String, name: String, detector_type: EdgeDetectorType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            detector_type,
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn detect(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(EdgeDetectionEvent::DetectionCompleted);

        match self.detector_type {
            EdgeDetectorType::Sobel => self.detect_sobel(input),
            EdgeDetectorType::Prewitt => self.detect_prewitt(input),
            EdgeDetectorType::Roberts => self.detect_roberts(input),
            EdgeDetectorType::Laplacian => self.detect_laplacian(input),
            EdgeDetectorType::Canny => self.detect_canny(input),
            EdgeDetectorType::Scharr => self.detect_scharr(input),
            EdgeDetectorType::FreiChen => self.detect_frei_chen(input),
            EdgeDetectorType::Kirsch => self.detect_kirsch(input),
            EdgeDetectorType::Robinson => self.detect_robinson(input),
            EdgeDetectorType::NevatiaBabu => self.detect_nevatia_babu(input),
            EdgeDetectorType::Custom(_) => self.detect_custom(input),
        }
    }

    fn detect_sobel(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let (gx_kernel, gy_kernel) = self.get_sobel_kernels(kernel_size);
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let gx_val = gx_pixel.luma();
                    let gy_val = gy_pixel.luma();
                    
                    let magnitude = (gx_val * gx_val + gy_val * gy_val).sqrt();
                    let edge_value = if normalize {
                        (magnitude / 255.0).min(1.0)
                    } else {
                        magnitude / 255.0
                    };
                    
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Sobel".to_string(),
            parameters: std::collections::HashMap::from([
                ("kernel_size".to_string(), kernel_size as f32),
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_prewitt(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let (gx_kernel, gy_kernel) = self.get_prewitt_kernels(kernel_size);
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let gx_val = gx_pixel.luma();
                    let gy_val = gy_pixel.luma();
                    
                    let magnitude = (gx_val * gx_val + gy_val * gy_val).sqrt();
                    let edge_value = if normalize {
                        (magnitude / 255.0).min(1.0)
                    } else {
                        magnitude / 255.0
                    };
                    
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Prewitt".to_string(),
            parameters: std::collections::HashMap::from([
                ("kernel_size".to_string(), kernel_size as f32),
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_roberts(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let (gx_kernel, gy_kernel) = self.get_roberts_kernels();
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let gx_val = gx_pixel.luma();
                    let gy_val = gy_pixel.luma();
                    
                    let magnitude = (gx_val * gx_val + gy_val * gy_val).sqrt();
                    let edge_value = if normalize {
                        (magnitude / 255.0).min(1.0)
                    } else {
                        magnitude / 255.0
                    };
                    
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Roberts".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_laplacian(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let kernel = self.get_laplacian_kernel(kernel_size);
        let laplacian = self.apply_convolution(input, &kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = laplacian.get_pixel(x, y) {
                    let laplacian_val = pixel.luma();
                    let edge_value = if normalize {
                        (laplacian_val / 255.0).abs().min(1.0)
                    } else {
                        (laplacian_val / 255.0).abs()
                    };
                    
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Laplacian".to_string(),
            parameters: std::collections::HashMap::from([
                ("kernel_size".to_string(), kernel_size as f32),
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_canny(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let low_threshold = parameters.get("low_threshold").copied().unwrap_or(0.05);
        let high_threshold = parameters.get("high_threshold").copied().unwrap_or(0.15);
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        let sigma = parameters.get("sigma").copied().unwrap_or(1.0);

Apply Gaussian blur first
        let blurred = self.apply_gaussian_blur(input, kernel_size, sigma)?;
        
        let (gx_kernel, gy_kernel) = self.get_sobel_kernels(3);
        let gx = self.apply_convolution(&blurred, &gx_kernel)?;
        let gy = self.apply_convolution(&blurred, &gy_kernel)?;
        
        let mut magnitude = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut direction = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let gx_val = gx_pixel.luma();
                    let gy_val = gy_pixel.luma();
                    
                    let mag = (gx_val * gx_val + gy_val * gy_val).sqrt();
                    let dir = gy_val.atan2(gx_val);
                    
                    let mag_pixel = crate::image_buffer::Pixel::gray(mag);
                    let dir_pixel = crate::image_buffer::Pixel::gray((dir + std::f32::consts::PI) * 255.0 / (2.0 * std::f32::consts::PI));
                    
                    magnitude.set_pixel(x, y, mag_pixel);
                    direction.set_pixel(x, y, dir_pixel);
                }
            }
        }
        
        let mut suppressed = self.non_maximum_suppression(&magnitude, &direction)?;
        
        let mut output = self.hysteresis_thresholding(&suppressed, low_threshold, high_threshold)?;
        
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..output.height {
            for x in 0..output.width {
                if let Some(pixel) = output.get_pixel(x, y) {
                    if pixel.luma() > 0.0 {
                        edge_count += 1;
                        total_strength += pixel.luma() / 255.0;
                    }
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Canny".to_string(),
            parameters: std::collections::HashMap::from([
                ("low_threshold".to_string(), low_threshold),
                ("high_threshold".to_string(), high_threshold),
                ("kernel_size".to_string(), kernel_size as f32),
                ("sigma".to_string(), sigma),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_scharr(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let (gx_kernel, gy_kernel) = self.get_scharr_kernels(kernel_size);
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let gx_val = gx_pixel.luma();
                    let gy_val = gy_pixel.luma();
                    
                    let magnitude = (gx_val * gx_val + gy_val * gy_val).sqrt();
                    let edge_value = if normalize {
                        (magnitude / 255.0).min(1.0)
                    } else {
                        magnitude / 255.0
                    };
                    
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Scharr".to_string(),
            parameters: std::collections::HashMap::from([
                ("kernel_size".to_string(), kernel_size as f32),
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_frei_chen(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let kernels = self.get_frei_chen_kernels();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut max_response = 0.0;
                
                for kernel in &kernels {
                    let response = self.apply_kernel_at_pixel(input, x, y, kernel);
                    max_response = max_response.max(response);
                }
                
                let edge_value = if normalize {
                    (max_response / 255.0).min(1.0)
                } else {
                    max_response / 255.0
                };
                
                let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                
                if final_value > 0.0 {
                    edge_count += 1;
                    total_strength += edge_value;
                }
                
                let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                output.set_pixel(x, y, edge_pixel);
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Frei-Chen".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_kirsch(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let kernels = self.get_kirsch_kernels();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut max_response = 0.0;
                
                for kernel in &kernels {
                    let response = self.apply_kernel_at_pixel(input, x, y, kernel);
                    max_response = max_response.max(response);
                }
                
                let edge_value = if normalize {
                    (max_response / 255.0).min(1.0)
                } else {
                    max_response / 255.0
                };
                
                let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                
                if final_value > 0.0 {
                    edge_count += 1;
                    total_strength += edge_value;
                }
                
                let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                output.set_pixel(x, y, edge_pixel);
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Kirsch".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_robinson(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let kernels = self.get_robinson_kernels();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut max_response = 0.0;
                
                for kernel in &kernels {
                    let response = self.apply_kernel_at_pixel(input, x, y, kernel);
                    max_response = max_response.max(response);
                }
                
                let edge_value = if normalize {
                    (max_response / 255.0).min(1.0)
                } else {
                    max_response / 255.0
                };
                
                let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                
                if final_value > 0.0 {
                    edge_count += 1;
                    total_strength += edge_value;
                }
                
                let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                output.set_pixel(x, y, edge_pixel);
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Robinson".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_nevatia_babu(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        let normalize = parameters.get("normalize").copied().unwrap_or(1.0) > 0.5;

        let kernels = self.get_nevatia_babu_kernels();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut max_response = 0.0;
                
                for kernel in &kernels {
                    let response = self.apply_kernel_at_pixel(input, x, y, kernel);
                    max_response = max_response.max(response);
                }
                
                let edge_value = if normalize {
                    (max_response / 255.0).min(1.0)
                } else {
                    max_response / 255.0
                };
                
                let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                
                if final_value > 0.0 {
                    edge_count += 1;
                    total_strength += edge_value;
                }
                
                let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                output.set_pixel(x, y, edge_pixel);
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Nevatia-Babu".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("normalize".to_string(), if normalize { 1.0 } else { 0.0 }),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn detect_custom(&self, input: &crate::image_buffer::ImageBuffer) -> Result<EdgeDetectionResult, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        let mut edge_count = 0u32;
        let mut total_strength = 0.0;
        
        for y in 1..(input.height - 1) {
            for x in 1..(input.width - 1) {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let center = pixel.luma();
                    let top = input.get_pixel(x, y - 1).map_or(0.0, |p| p.luma());
                    let bottom = input.get_pixel(x, y + 1).map_or(0.0, |p| p.luma());
                    let left = input.get_pixel(x - 1, y).map_or(0.0, |p| p.luma());
                    let right = input.get_pixel(x + 1, y).map_or(0.0, |p| p.luma());
                    
                    let gradient = ((center - top).abs() + (center - bottom).abs() + 
                                   (center - left).abs() + (center - right).abs()) / 4.0;
                    
                    let edge_value = gradient / 255.0;
                    let final_value = if edge_value > threshold { 255.0 } else { 0.0 };
                    
                    if final_value > 0.0 {
                        edge_count += 1;
                        total_strength += edge_value;
                    }
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        let avg_strength = if edge_count > 0 {
            total_strength / edge_count as f32
        } else {
            0.0
        };

        Ok(EdgeDetectionResult {
            edge_image: output,
            method: "Custom".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
            ]),
            edge_count,
            edge_strength: avg_strength,
        })
    }

    fn get_sobel_kernels(&self, size: u32) -> (Vec<f32>, Vec<f32>) {
        match size {
            3 => (
                vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0],
                vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0],
            ),
            5 => {
                let gx = vec![
                    -1.0, -2.0, 0.0, 2.0, 1.0,
                    -4.0, -8.0, 0.0, 8.0, 4.0,
                    -6.0, -12.0, 0.0, 12.0, 6.0,
                    -4.0, -8.0, 0.0, 8.0, 4.0,
                    -1.0, -2.0, 0.0, 2.0, 1.0,
                ];
                let gy = vec![
                    -1.0, -4.0, -6.0, -4.0, -1.0,
                    -2.0, -8.0, -12.0, -8.0, -2.0,
                    0.0, 0.0, 0.0, 0.0, 0.0,
                    2.0, 8.0, 12.0, 8.0, 2.0,
                    1.0, 4.0, 6.0, 4.0, 1.0,
                ];
                (gx, gy)
            },
            _ => self.get_sobel_kernels(3),
        }
    }

    fn get_prewitt_kernels(&self, size: u32) -> (Vec<f32>, Vec<f32>) {
        match size {
            3 => (
                vec![-1.0, 0.0, 1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0],
                vec![-1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            ),
            _ => self.get_prewitt_kernels(3),
        }
    }

    fn get_roberts_kernels(&self) -> (Vec<f32>, Vec<f32>) {
        (
            vec![1.0, 0.0, 0.0, -1.0],
            vec![0.0, 1.0, -1.0, 0.0],
        )
    }

    fn get_laplacian_kernel(&self, size: u32) -> Vec<f32> {
        match size {
            3 => vec![0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0],
            5 => {
                vec![
                    0.0, 0.0, 1.0, 0.0, 0.0,
                    0.0, 1.0, 2.0, 1.0, 0.0,
                    1.0, 2.0, -16.0, 2.0, 1.0,
                    0.0, 1.0, 2.0, 1.0, 0.0,
                    0.0, 0.0, 1.0, 0.0, 0.0,
                ]
            },
            _ => self.get_laplacian_kernel(3),
        }
    }

    fn get_scharr_kernels(&self, size: u32) -> (Vec<f32>, Vec<f32>) {
        match size {
            3 => (
                vec![-3.0, 0.0, 3.0, -10.0, 0.0, 10.0, -3.0, 0.0, 3.0],
                vec![-3.0, -10.0, -3.0, 0.0, 0.0, 0.0, 3.0, 10.0, 3.0],
            ),
            _ => self.get_scharr_kernels(3),
        }
    }

    fn get_frei_chen_kernels(&self) -> Vec<Vec<f32>> {
        vec![
            vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, -1.0],
            vec![1.0, 0.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, -1.0],
            vec![0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0],
            vec![1.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, -1.0, -1.0],
            vec![1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0],
            vec![0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0],
        ]
    }

    fn get_kirsch_kernels(&self) -> Vec<Vec<f32>> {
        vec![
            vec![-3.0, -3.0, 5.0, -3.0, 0.0, 5.0, -3.0, -3.0, 5.0],
            vec![-3.0, 5.0, 5.0, -3.0, 0.0, 5.0, -3.0, -3.0, -3.0],
            vec![5.0, 5.0, 5.0, -3.0, 0.0, -3.0, -3.0, -3.0, -3.0],
            vec![5.0, 5.0, -3.0, 5.0, 0.0, -3.0, -3.0, -3.0, -3.0],
            vec![5.0, -3.0, -3.0, 5.0, 0.0, -3.0, 5.0, -3.0, -3.0],
            vec![-3.0, -3.0, -3.0, 5.0, 0.0, -3.0, 5.0, 5.0, -3.0],
            vec![-3.0, -3.0, -3.0, -3.0, 0.0, -3.0, 5.0, 5.0, 5.0],
            vec![-3.0, -3.0, 5.0, -3.0, 0.0, 5.0, -3.0, -3.0, 5.0],
        ]
    }

    fn get_robinson_kernels(&self) -> Vec<Vec<f32>> {
        vec![
            vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0],
            vec![0.0, 1.0, 2.0, -1.0, 0.0, 1.0, -2.0, -1.0, 0.0],
            vec![1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0],
            vec![2.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, -1.0, -2.0],
            vec![1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0],
            vec![0.0, -1.0, -2.0, 1.0, 0.0, -1.0, 2.0, 1.0, 0.0],
            vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0],
            vec![-2.0, -1.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 2.0],
        ]
    }

    fn get_nevatia_babu_kernels(&self) -> Vec<Vec<f32>> {
        vec![
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
            vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
        ]
    }

    fn apply_convolution(&self, input: &crate::image_buffer::ImageBuffer, kernel: &[f32]) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel_size = (kernel.len() as f32).sqrt() as u32;
        let half_kernel = kernel_size / 2;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut sum = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let src_x = (x as i32 + kx as i32 - half_kernel as i32).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + ky as i32 - half_kernel as i32).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            let weight = kernel[(ky * kernel_size + kx) as usize];
                            sum += pixel.luma() * weight;
                        }
                    }
                }
                
                let convolved_pixel = crate::image_buffer::Pixel::gray(sum.clamp(0.0, 255.0));
                output.set_pixel(x, y, convolved_pixel);
            }
        }
        
        Ok(output)
    }

    fn apply_kernel_at_pixel(&self, input: &crate::image_buffer::ImageBuffer, x: u32, y: u32, kernel: &[f32]) -> f32 {
        let kernel_size = (kernel.len() as f32).sqrt() as u32;
        let half_kernel = kernel_size / 2;
        
        let mut sum = 0.0;
        
        for ky in 0..kernel_size {
            for kx in 0..kernel_size {
                let src_x = (x as i32 + kx as i32 - half_kernel as i32).clamp(0, input.width as i32 - 1) as u32;
                let src_y = (y as i32 + ky as i32 - half_kernel as i32).clamp(0, input.height as i32 - 1) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    let weight = kernel[(ky * kernel_size + kx) as usize];
                    sum += pixel.luma() * weight;
                }
            }
        }
        
        sum
    }

    fn apply_gaussian_blur(&self, input: &crate::image_buffer::ImageBuffer, kernel_size: u32, sigma: f32) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = self.gaussian_kernel(kernel_size, sigma);
        self.apply_convolution(input, &kernel)
    }

    fn gaussian_kernel(&self, size: u32, sigma: f32) -> Vec<f32> {
        let mut kernel = Vec::with_capacity((size * size) as usize);
        let center = size as f32 / 2.0;
        let sum = 0.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let value = (- (dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                kernel.push(value);
            }
        }
        
        let kernel_sum: f32 = kernel.iter().sum();
        kernel.iter_mut().for_each(|v| *v /= kernel_sum);
        
        kernel
    }

    fn non_maximum_suppression(&self, magnitude: &crate::image_buffer::ImageBuffer, direction: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = crate::image_buffer::ImageBuffer::new(magnitude.width, magnitude.height, magnitude.pixel_format.clone());
        
        for y in 1..(magnitude.height - 1) {
            for x in 1..(magnitude.width - 1) {
                if let (Some(mag_pixel), Some(dir_pixel)) = (magnitude.get_pixel(x, y), direction.get_pixel(x, y)) {
                    let mag = mag_pixel.luma();
                    let dir = dir_pixel.luma() / 255.0 * 8.0;
                    
                    let (dx, dy) = match dir as i32 {
                        0..1 => (1, 0),
                        1..2 => (1, 1),
                        2..3 => (0, 1),
                        3..4 => (-1, 1),
                        4..5 => (-1, 0),
                        5..6 => (-1, -1),
                        6..7 => (0, -1),
                        _ => (1, -1),
                    };
                    
                    let neighbor1 = magnitude.get_pixel(
                        (x as i32 + dx).clamp(0, magnitude.width as i32 - 1) as u32,
                        (y as i32 + dy).clamp(0, magnitude.height as i32 - 1) as u32,
                    ).map_or(0.0, |p| p.luma());
                    
                    let neighbor2 = magnitude.get_pixel(
                        (x as i32 - dx).clamp(0, magnitude.width as i32 - 1) as u32,
                        (y as i32 - dy).clamp(0, magnitude.height as i32 - 1) as u32,
                    ).map_or(0.0, |p| p.luma());
                    
                    let suppressed_value = if mag >= neighbor1 && mag >= neighbor2 {
                        mag
                    } else {
                        0.0
                    };
                    
                    let suppressed_pixel = crate::image_buffer::Pixel::gray(suppressed_value);
                    output.set_pixel(x, y, suppressed_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn hysteresis_thresholding(&self, input: &crate::image_buffer::ImageBuffer, low_threshold: f32, high_threshold: f32) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let value = pixel.luma() / 255.0;
                    let thresholded_value = if value > high_threshold {
                        255.0
                    } else if value > low_threshold {
                        128.0
                    } else {
                        0.0
                    };
                    
                    let thresholded_pixel = crate::image_buffer::Pixel::gray(thresholded_value);
                    output.set_pixel(x, y, thresholded_pixel);
                }
            }
        }
        
        Ok(output)
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(EdgeDetectionEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub async fn get_events(&mut self) -> Vec<EdgeDetectionEvent> {
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

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub fn clone_detector(&self) -> EdgeDetector {
        let mut new_detector = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.detector_type.clone(),
        );
        
        let parameters = self.parameters.read();
        *new_detector.parameters.write() = parameters.clone();
        
        new_detector
    }
}

impl Default for EdgeDetector {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Edge Detector".to_string(),
            EdgeDetectorType::Sobel,
        )
    }
}

impl Default for EdgeDetectorType {
    fn default() -> Self {
        EdgeDetectorType::Sobel
    }
}

impl Default for EdgeDetectionResult {
    fn default() -> Self {
        let image = crate::image_buffer::ImageBuffer::default();
        Self {
            edge_image: image,
            method: "Sobel".to_string(),
            parameters: std::collections::HashMap::new(),
            edge_count: 0,
            edge_strength: 0.0,
        }
    }
}
