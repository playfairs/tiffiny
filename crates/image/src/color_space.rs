use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    RGB,
    HSV,
    HSL,
    HSI,
    HSB,
    Lab,
    LCH,
    XYZ,
    YUV,
    CMYK,
    Gray,
}

#[derive(Debug, Clone)]
pub struct ColorSpaceConverter {
    pub id: String,
    pub input_space: Arc<RwLock<Option<ColorSpace>>>,
    pub output_space: Arc<RwLock<Option<ColorSpace>>>,
    pub event_sender: mpsc::UnboundedSender<ColorSpaceEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ColorSpaceEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ColorSpaceEvent {
    InputSpaceChanged(ColorSpace),
    OutputSpaceChanged(ColorSpace),
    ConversionCompleted,
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct RGBColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HSVColor {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HSLColor {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LabColor {
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct XYZColor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct YUVColor {
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CMYKColor {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub alpha: f32,
}

impl ColorSpaceConverter {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_space: Arc::new(RwLock::new(None)),
            output_space: Arc::new(RwLock::new(None)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn set_input_space(&self, space: ColorSpace) {
        let mut input_space = self.input_space.write();
        *input_space = Some(space.clone());
        
        let _ = self.event_sender.send(ColorSpaceEvent::InputSpaceChanged(space));
    }

    pub fn set_output_space(&self, space: ColorSpace) {
        let mut output_space = self.output_space.write();
        *output_space = Some(space.clone());
        
        let _ = self.event_sender.send(ColorSpaceEvent::OutputSpaceChanged(space));
    }

    pub async fn convert_pixel(&self, pixel: crate::image_buffer::Pixel) -> Result<crate::image_buffer::Pixel, Box<dyn std::error::Error>> {
        let input_space = self.input_space.read();
        let output_space = self.output_space.read();
        
        if let (Some(input), Some(output)) = (input_space.as_ref(), output_space.as_ref()) {
            if input == output {
                return Ok(pixel);
            }
            
            let result = match (input, output) {
                (ColorSpace::RGB, ColorSpace::HSV) => self.rgb_to_hsv(pixel),
                (ColorSpace::RGB, ColorSpace::HSL) => self.rgb_to_hsl(pixel),
                (ColorSpace::RGB, ColorSpace::Lab) => self.rgb_to_lab(pixel),
                (ColorSpace::RGB, ColorSpace::XYZ) => self.rgb_to_xyz(pixel),
                (ColorSpace::RGB, ColorSpace::YUV) => self.rgb_to_yuv(pixel),
                (ColorSpace::RGB, ColorSpace::CMYK) => self.rgb_to_cmyk(pixel),
                (ColorSpace::RGB, ColorSpace::Gray) => self.rgb_to_gray(pixel),
                (ColorSpace::HSV, ColorSpace::RGB) => self.hsv_to_rgb(pixel),
                (ColorSpace::HSL, ColorSpace::RGB) => self.hsl_to_rgb(pixel),
                (ColorSpace::Lab, ColorSpace::RGB) => self.lab_to_rgb(pixel),
                (ColorSpace::XYZ, ColorSpace::RGB) => self.xyz_to_rgb(pixel),
                (ColorSpace::YUV, ColorSpace::RGB) => self.yuv_to_rgb(pixel),
                (ColorSpace::CMYK, ColorSpace::RGB) => self.cmyk_to_rgb(pixel),
                (ColorSpace::Gray, ColorSpace::RGB) => self.gray_to_rgb(pixel),
                _ => {
                    let error_msg = format!("Unsupported conversion from {:?} to {:?}", input, output);
                    let _ = self.event_sender.send(ColorSpaceEvent::Error(error_msg));
                    return Err(error_msg.into());
                },
            };
            
            let _ = self.event_sender.send(ColorSpaceEvent::ConversionCompleted);
            Ok(result)
        } else {
            Err("Input or output color space not set".into())
        }
    }

    pub async fn convert_image(&self, image: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let input_space = self.input_space.read();
        let output_space = self.output_space.read();
        
        if let (Some(input), Some(output)) = (input_space.as_ref(), output_space.as_ref()) {
            let mut output_image = crate::image_buffer::ImageBuffer::new(image.width, image.height, image.pixel_format.clone());
            
            for y in 0..image.height {
                for x in 0..image.width {
                    if let Some(pixel) = image.get_pixel(x, y) {
                        let converted_pixel = self.convert_pixel(pixel).await?;
                        output_image.set_pixel(x, y, converted_pixel);
                    }
                }
            }
            
            Ok(output_image)
        } else {
            Err("Input or output color space not set".into())
        }
    }

    pub fn get_input_space(&self) -> Option<ColorSpace> {
        self.input_space.read().clone()
    }

    pub fn get_output_space(&self) -> Option<ColorSpace> {
        self.output_space.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<ColorSpaceEvent> {
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

RGB to other color spaces
    fn rgb_to_hsv(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let r = rgb.r / 255.0;
        let g = rgb.g / 255.0;
        let b = rgb.b / 255.0;
        
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta)
        } else if max == g {
            60.0 * ((b - r) / delta) + 120.0
        } else {
            60.0 * ((r - g) / delta) + 240.0
        };
        
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;
        
        crate::image_buffer::Pixel {
            r: h.clamp(0.0, 360.0) * 255.0 / 360.0,
            g: s.clamp(0.0, 1.0) * 255.0,
            b: v.clamp(0.0, 1.0) * 255.0,
            a: rgb.a,
        }
    }

    fn rgb_to_hsl(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let r = rgb.r / 255.0;
        let g = rgb.g / 255.0;
        let b = rgb.b / 255.0;
        
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let sum = r + g + b;
        
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta)
        } else if max == g {
            60.0 * ((b - r) / delta) + 120.0
        } else {
            60.0 * ((r - g) / delta) + 240.0
        };
        
        let l = sum / 3.0;
        let s = if delta == 0.0 { 0.0 } else { 
            if l <= 0.5 {
                delta / (sum)
            } else {
                delta / (3.0 - sum)
            }
        };
        
        crate::image_buffer::Pixel {
            r: h.clamp(0.0, 360.0) * 255.0 / 360.0,
            g: s.clamp(0.0, 1.0) * 255.0,
            b: l.clamp(0.0, 1.0) * 255.0,
            a: rgb.a,
        }
    }

    fn rgb_to_lab(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let xyz = self.rgb_to_xyz_internal(rgb.r, rgb.g, rgb.b);
        
        let lab = self.xyz_to_lab_internal(xyz.x, xyz.y, xyz.z);
        
        crate::image_buffer::Pixel {
            r: lab.l.clamp(0.0, 100.0) * 2.55,
            g: (lab.a + 128.0).clamp(0.0, 255.0),
            b: (lab.b + 128.0).clamp(0.0, 255.0),
            a: rgb.a,
        }
    }

    fn rgb_to_xyz(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let xyz = self.rgb_to_xyz_internal(rgb.r, rgb.g, rgb.b);
        
        crate::image_buffer::Pixel {
            r: xyz.x.clamp(0.0, 100.0) * 2.55,
            g: xyz.y.clamp(0.0, 100.0) * 2.55,
            b: xyz.z.clamp(0.0, 100.0) * 2.55,
            a: rgb.a,
        }
    }

    fn rgb_to_yuv(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let r = rgb.r / 255.0;
        let g = rgb.g / 255.0;
        let b = rgb.b / 255.0;
        
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
        let v = 0.615 * r - 0.51499 * g - 0.10001 * b;
        
        crate::image_buffer::Pixel {
            r: y.clamp(0.0, 1.0) * 255.0,
            g: (u + 0.5) * 255.0,
            b: (v + 0.5) * 255.0,
            a: rgb.a,
        }
    }

    fn rgb_to_cmyk(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let r = rgb.r / 255.0;
        let g = rgb.g / 255.0;
        let b = rgb.b / 255.0;
        
        let k = 1.0 - r.max(g).max(b);
        let c = (1.0 - r - k) / (1.0 - k);
        let m = (1.0 - g - k) / (1.0 - k);
        let y = (1.0 - b - k) / (1.0 - k);
        
        crate::image_buffer::Pixel {
            r: c.clamp(0.0, 1.0) * 255.0,
            g: m.clamp(0.0, 1.0) * 255.0,
            b: y.clamp(0.0, 1.0) * 255.0,
            a: k.clamp(0.0, 1.0) * 255.0,
        }
    }

    fn rgb_to_gray(&self, rgb: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let gray = (0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b).round();
        
        crate::image_buffer::Pixel {
            r: gray,
            g: gray,
            b: gray,
            a: rgb.a,
        }
    }

    fn hsv_to_rgb(&self, hsv: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let h = hsv.r / 255.0 * 360.0 / 60.0;
        let s = hsv.g / 255.0;
        let v = hsv.b / 255.0;
        
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        crate::image_buffer::Pixel {
            r: ((r + m) * 255.0).clamp(0.0, 255.0),
            g: ((g + m) * 255.0).clamp(0.0, 255.0),
            b: ((b + m) * 255.0).clamp(0.0, 255.0),
            a: hsv.a,
        }
    }

    fn hsl_to_rgb(&self, hsl: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let h = hsl.r / 255.0 * 360.0 / 60.0;
        let s = hsl.g / 255.0;
        let l = hsl.b / 255.0;
        
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = l - c / 2.0;
        
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        crate::image_buffer::Pixel {
            r: ((r + m) * 255.0).clamp(0.0, 255.0),
            g: ((g + m) * 255.0).clamp(0.0, 255.0),
            b: ((b + m) * 255.0).clamp(0.0, 255.0),
            a: hsl.a,
        }
    }

    fn lab_to_rgb(&self, lab: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let l = lab.r / 2.55;
        let a = lab.g - 128.0;
        let b = lab.b - 128.0;
        
        let xyz = self.lab_to_xyz_internal(l, a, b);
        
        let rgb = self.xyz_to_rgb_internal(xyz.x, xyz.y, xyz.z);
        
        crate::image_buffer::Pixel {
            r: rgb.r.clamp(0.0, 255.0),
            g: rgb.g.clamp(0.0, 255.0),
            b: rgb.b.clamp(0.0, 255.0),
            a: lab.a,
        }
    }

    fn xyz_to_rgb(&self, xyz: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let x = xyz.r / 2.55;
        let y = xyz.g / 2.55;
        let z = xyz.b / 2.55;
        
        let rgb = self.xyz_to_rgb_internal(x, y, z);
        
        crate::image_buffer::Pixel {
            r: rgb.r.clamp(0.0, 255.0),
            g: rgb.g.clamp(0.0, 255.0),
            b: rgb.b.clamp(0.0, 255.0),
            a: xyz.a,
        }
    }

    fn yuv_to_rgb(&self, yuv: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let y = yuv.r / 255.0;
        let u = yuv.g / 255.0 - 0.5;
        let v = yuv.b / 255.0 - 0.5;
        
        let r = y + 1.13983 * v;
        let g = y - 0.39465 * u - 0.58060 * v;
        let b = y + 2.03211 * u;
        
        crate::image_buffer::Pixel {
            r: r.clamp(0.0, 1.0) * 255.0,
            g: g.clamp(0.0, 1.0) * 255.0,
            b: b.clamp(0.0, 1.0) * 255.0,
            a: yuv.a,
        }
    }

    fn cmyk_to_rgb(&self, cmyk: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        let c = cmyk.r / 255.0;
        let m = cmyk.g / 255.0;
        let y = cmyk.b / 255.0;
        let k = cmyk.a / 255.0;
        
        let r = 255.0 * (1.0 - c) * (1.0 - k);
        let g = 255.0 * (1.0 - m) * (1.0 - k);
        let b = 255.0 * (1.0 - y) * (1.0 - k);
        
        crate::image_buffer::Pixel {
            r: r.clamp(0.0, 255.0),
            g: g.clamp(0.0, 255.0),
            b: b.clamp(0.0, 255.0),
            a: 255.0,
        }
    }

    fn gray_to_rgb(&self, gray: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        crate::image_buffer::Pixel {
            r: gray.r,
            g: gray.r,
            b: gray.r,
            a: gray.a,
        }
    }

    fn rgb_to_xyz_internal(&self, r: f32, g: f32, b: f32) -> XYZColor {
        let r_norm = r / 255.0;
        let g_norm = g / 255.0;
        let b_norm = b / 255.0;
        
        let r_linear = if r_norm <= 0.04045 {
            r_norm / 12.92
        } else {
            ((r_norm + 0.055) / 1.055).powf(2.4)
        };
        
        let g_linear = if g_norm <= 0.04045 {
            g_norm / 12.92
        } else {
            ((g_norm + 0.055) / 1.055).powf(2.4)
        };
        
        let b_linear = if b_norm <= 0.04045 {
            b_norm / 12.92
        } else {
            ((b_norm + 0.055) / 1.055).powf(2.4)
        };
        
        let x = r_linear * 0.4124564 + g_linear * 0.3575761 + b_linear * 0.1804375;
        let y = r_linear * 0.2126729 + g_linear * 0.7151522 + b_linear * 0.0721750;
        let z = r_linear * 0.0193339 + g_linear * 0.1191920 + b_linear * 0.9503041;
        
        XYZColor {
            x: x * 100.0,
            y: y * 100.0,
            z: z * 100.0,
            alpha: 255.0,
        }
    }

    fn xyz_to_lab_internal(&self, x: f32, y: f32, z: f32) -> LabColor {
        let x_n = x / 95.047;
        let y_n = y / 100.0;
        let z_n = z / 108.883;
        
        let fx = if x_n > 0.008856 {
            x_n.powf(1.0 / 3.0)
        } else {
            7.787 * x_n + 16.0 / 116.0
        };
        
        let fy = if y_n > 0.008856 {
            y_n.powf(1.0 / 3.0)
        } else {
            7.787 * y_n + 16.0 / 116.0
        };
        
        let fz = if z_n > 0.008856 {
            z_n.powf(1.0 / 3.0)
        } else {
            7.787 * z_n + 16.0 / 116.0
        };
        
        let l = 116.0 * fy - 16.0;
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);
        
        LabColor {
            l: l.clamp(0.0, 100.0),
            a: a.clamp(-128.0, 127.0),
            b: b.clamp(-128.0, 127.0),
            alpha: 255.0,
        }
    }

    fn lab_to_xyz_internal(&self, l: f32, a: f32, b: f32) -> XYZColor {
        let fy = (l + 16.0) / 116.0;
        let fx = a / 500.0 + fy;
        let fz = b / 200.0 + fy;
        
        let x_n = if fx.powf(3.0) > 0.008856 {
            fx.powf(3.0)
        } else {
            (fx - 16.0 / 116.0) / 7.787
        };
        
        let y_n = if fy.powf(3.0) > 0.008856 {
            fy.powf(3.0)
        } else {
            (fy - 16.0 / 116.0) / 7.787
        };
        
        let z_n = if fz.powf(3.0) > 0.008856 {
            fz.powf(3.0)
        } else {
            (fz - 16.0 / 116.0) / 7.787
        };
        
        let x = x_n * 95.047;
        let y = y_n * 100.0;
        let z = z_n * 108.883;
        
        XYZColor {
            x: x.clamp(0.0, 100.0),
            y: y.clamp(0.0, 100.0),
            z: z.clamp(0.0, 100.0),
            alpha: 255.0,
        }
    }

    fn xyz_to_rgb_internal(&self, x: f32, y: f32, z: f32) -> RGBColor {
        let x_norm = x / 100.0;
        let y_norm = y / 100.0;
        let z_norm = z / 100.0;
        
        let r_linear = x_norm * 3.2404542 - y_norm * 1.5371385 - z_norm * 0.4985314;
        let g_linear = -x_norm * 0.9692660 + y_norm * 1.8760108 + z_norm * 0.0415560;
        let b_linear = x_norm * 0.0556434 - y_norm * 0.2040259 + z_norm * 1.0572252;
        
        let r = if r_linear <= 0.0031308 {
            12.92 * r_linear
        } else {
            1.055 * r_linear.powf(1.0 / 2.4) - 0.055
        };
        
        let g = if g_linear <= 0.0031308 {
            12.92 * g_linear
        } else {
            1.055 * g_linear.powf(1.0 / 2.4) - 0.055
        };
        
        let b = if b_linear <= 0.0031308 {
            12.92 * b_linear
        } else {
            1.055 * b_linear.powf(1.0 / 2.4) - 0.055
        };
        
        RGBColor {
            r: (r * 255.0).clamp(0.0, 255.0),
            g: (g * 255.0).clamp(0.0, 255.0),
            b: (b * 255.0).clamp(0.0, 255.0),
            a: 255.0,
        }
    }

    pub fn get_supported_conversions(&self) -> Vec<(ColorSpace, ColorSpace)> {
        vec![
            (ColorSpace::RGB, ColorSpace::HSV),
            (ColorSpace::RGB, ColorSpace::HSL),
            (ColorSpace::RGB, ColorSpace::Lab),
            (ColorSpace::RGB, ColorSpace::XYZ),
            (ColorSpace::RGB, ColorSpace::YUV),
            (ColorSpace::RGB, ColorSpace::CMYK),
            (ColorSpace::RGB, ColorSpace::Gray),
            (ColorSpace::HSV, ColorSpace::RGB),
            (ColorSpace::HSL, ColorSpace::RGB),
            (ColorSpace::Lab, ColorSpace::RGB),
            (ColorSpace::XYZ, ColorSpace::RGB),
            (ColorSpace::YUV, ColorSpace::RGB),
            (ColorSpace::CMYK, ColorSpace::RGB),
            (ColorSpace::Gray, ColorSpace::RGB),
        ]
    }

    pub fn is_conversion_supported(&self, from: &ColorSpace, to: &ColorSpace) -> bool {
        self.get_supported_conversions().contains(&(*from, *to))
    }

    pub fn get_conversion_info(&self) -> ConversionInfo {
        let input_space = self.input_space.read();
        let output_space = self.output_space.read();
        
        ConversionInfo {
            input_space: input_space.clone(),
            output_space: output_space.clone(),
            conversion_type: self.get_conversion_type(&input_space, &output_space),
            is_lossless: self.is_lossless_conversion(&input_space, &output_space),
        }
    }

    fn get_conversion_type(&self, input: &Option<ColorSpace>, output: &Option<ColorSpace>) -> ConversionType {
        match (input, output) {
            (Some(ColorSpace::RGB), Some(ColorSpace::HSV)) => ConversionType::RGBToHSV,
            (Some(ColorSpace::RGB), Some(ColorSpace::HSL)) => ConversionType::RGBToHSL,
            (Some(ColorSpace::RGB), Some(ColorSpace::Lab)) => ConversionType::RGBToLab,
            (Some(ColorSpace::RGB), Some(ColorSpace::XYZ)) => ConversionType::RGBToXYZ,
            (Some(ColorSpace::RGB), Some(ColorSpace::YUV)) => ConversionType::RGBToYUV,
            (Some(ColorSpace::RGB), Some(ColorSpace::CMYK)) => ConversionType::RGBToCMYK,
            (Some(ColorSpace::RGB), Some(ColorSpace::Gray)) => ConversionType::RGBToGray,
            (Some(ColorSpace::HSV), Some(ColorSpace::RGB)) => ConversionType::HSVToRGB,
            (Some(ColorSpace::HSL), Some(ColorSpace::RGB)) => ConversionType::HSLToRGB,
            (Some(ColorSpace::Lab), Some(ColorSpace::RGB)) => ConversionType::LabToRGB,
            (Some(ColorSpace::XYZ), Some(ColorSpace::RGB)) => ConversionType::XYZToRGB,
            (Some(ColorSpace::YUV), Some(ColorSpace::RGB)) => ConversionType::YUVToRGB,
            (Some(ColorSpace::CMYK), Some(ColorSpace::RGB)) => ConversionType::CMYKToRGB,
            (Some(ColorSpace::Gray), Some(ColorSpace::RGB)) => ConversionType::GrayToRGB,
            _ => ConversionType::Unknown,
        }
    }

    fn is_lossless_conversion(&self, input: &Option<ColorSpace>, output: &Option<ColorSpace>) -> bool {
        match (input, output) {
            (Some(ColorSpace::RGB), Some(ColorSpace::Gray)) => false,
            (Some(ColorSpace::Gray), Some(ColorSpace::RGB)) => false,
            (Some(ColorSpace::RGB), Some(ColorSpace::CMYK)) => true,
            (Some(ColorSpace::CMYK), Some(ColorSpace::RGB)) => true,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionInfo {
    pub input_space: Option<ColorSpace>,
    pub output_space: Option<ColorSpace>,
    pub conversion_type: ConversionType,
    pub is_lossless: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionType {
    RGBToHSV,
    RGBToHSL,
    RGBToLab,
    RGBToXYZ,
    RGBToYUV,
    RGBToCMYK,
    RGBToGray,
    HSVToRGB,
    HSLToRGB,
    LabToRGB,
    XYZToRGB,
    YUVToRGB,
    CMYKToRGB,
    GrayToRGB,
    Unknown,
}

impl Default for ColorSpaceConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::RGB
    }
}

impl Default for ConversionInfo {
    fn default() -> Self {
        Self {
            input_space: None,
            output_space: None,
            conversion_type: ConversionType::Unknown,
            is_lossless: true,
        }
    }
}
