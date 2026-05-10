use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ChecksumCalculator {
    pub id: String,
    pub name: String,
    pub algorithm: Arc<RwLock<ChecksumAlgorithm>>,
    pub event_sender: mpsc::UnboundedSender<ChecksumEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ChecksumEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChecksumAlgorithm {
    CRC8,
    CRC16,
    CRC32,
    CRC64,
    MD5,
    SHA1,
    SHA256,
    SHA512,
    Adler32,
    Fletcher16,
    Fletcher32,
    Fletcher64,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ChecksumEvent {
    CalculationStarted,
    CalculationProgress(f32),
    CalculationCompleted(ChecksumResult),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ChecksumResult {
    pub algorithm: String,
    pub checksum: String,
    pub hex_checksum: String,
    pub bytes_processed: u64,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChecksumConfig {
    pub algorithm: ChecksumAlgorithm,
    pub chunk_size: usize,
    pub use_lowercase: bool,
    pub include_prefix: bool,
    pub custom_polynomial: Option<u64>,
    pub custom_initial_value: Option<u64>,
}

impl ChecksumCalculator {
    pub fn new(id: String, name: String, algorithm: ChecksumAlgorithm) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            algorithm: Arc::new(RwLock::new(algorithm)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn calculate(&self, data: &[u8], config: ChecksumConfig) -> Result<ChecksumResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ChecksumEvent::CalculationStarted);
        let start_time = std::time::Instant::now();

        let result = match config.algorithm {
            ChecksumAlgorithm::CRC8 => self.calculate_crc8(data, &config),
            ChecksumAlgorithm::CRC16 => self.calculate_crc16(data, &config),
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data, &config),
            ChecksumAlgorithm::CRC64 => self.calculate_crc64(data, &config),
            ChecksumAlgorithm::MD5 => self.calculate_md5(data, &config),
            ChecksumAlgorithm::SHA1 => self.calculate_sha1(data, &config),
            ChecksumAlgorithm::SHA256 => self.calculate_sha256(data, &config),
            ChecksumAlgorithm::SHA512 => self.calculate_sha512(data, &config),
            ChecksumAlgorithm::Adler32 => self.calculate_adler32(data, &config),
            ChecksumAlgorithm::Fletcher16 => self.calculate_fletcher16(data, &config),
            ChecksumAlgorithm::Fletcher32 => self.calculate_fletcher32(data, &config),
            ChecksumAlgorithm::Fletcher64 => self.calculate_fletcher64(data, &config),
            ChecksumAlgorithm::Custom(_) => self.calculate_custom(data, &config),
        };

        let processing_time = start_time.elapsed();
        let bytes_processed = data.len() as u64;

        match result {
            Ok(checksum) => {
                let checksum_result = ChecksumResult {
                    algorithm: format!("{:?}", config.algorithm),
                    checksum: checksum.clone(),
                    hex_checksum: checksum,
                    bytes_processed,
                    processing_time,
                    error_message: None,
                };

                let _ = self.event_sender.send(ChecksumEvent::CalculationCompleted(checksum_result.clone()));
                Ok(checksum_result)
            },
            Err(e) => {
                let error_msg = format!("Checksum calculation failed: {}", e);
                let _ = self.event_sender.send(ChecksumEvent::Error(error_msg.clone()));

                Ok(ChecksumResult {
                    algorithm: format!("{:?}", config.algorithm),
                    checksum: String::new(),
                    hex_checksum: String::new(),
                    bytes_processed,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    pub async fn calculate_file(&self, file_path: &str, config: ChecksumConfig) -> Result<ChecksumResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ChecksumEvent::CalculationStarted);
        let start_time = std::time::Instant::now();

Read file in chunks
        let mut file = std::fs::File::open(file_path)?;
        let file_size = file.metadata()?.len();
        let mut bytes_processed = 0u64;

        let mut checksum_state = self.initialize_checksum_state(&config);

        let mut buffer = vec![0u8; config.chunk_size];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            self.update_checksum(&mut checksum_state, &buffer[..bytes_read], &config);
            bytes_processed += bytes_read as u64;

            let progress = (bytes_processed as f32 / file_size as f32) * 100.0;
            let _ = self.event_sender.send(ChecksumEvent::CalculationProgress(progress));
        }

        let checksum = self.finalize_checksum(checksum_state, &config);
        let processing_time = start_time.elapsed();

        let checksum_result = ChecksumResult {
            algorithm: format!("{:?}", config.algorithm),
            checksum: checksum.clone(),
            hex_checksum: checksum,
            bytes_processed,
            processing_time,
            error_message: None,
        };

        let _ = self.event_sender.send(ChecksumEvent::CalculationCompleted(checksum_result.clone()));
        Ok(checksum_result)
    }

    pub async fn calculate_with_progress<F>(&self, data: &[u8], config: ChecksumConfig, progress_callback: F) -> Result<ChecksumResult, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(ChecksumEvent::CalculationStarted);
        let start_time = std::time::Instant::now();

        let chunk_size = config.chunk_size;
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;

        let mut checksum_state = self.initialize_checksum_state(&config);

        for (chunk_index, chunk) in data.chunks(chunk_size).enumerate() {
            self.update_checksum(&mut checksum_state, chunk, &config);

            let progress = (chunk_index as f32 / total_chunks as f32) * 100.0;
            progress_callback(progress);
            let _ = self.event_sender.send(ChecksumEvent::CalculationProgress(progress));
        }

        let checksum = self.finalize_checksum(checksum_state, &config);
        let processing_time = start_time.elapsed();
        let bytes_processed = data.len() as u64;

        let checksum_result = ChecksumResult {
            algorithm: format!("{:?}", config.algorithm),
            checksum: checksum.clone(),
            hex_checksum: checksum,
            bytes_processed,
            processing_time,
            error_message: None,
        };

        let _ = self.event_sender.send(ChecksumEvent::CalculationCompleted(checksum_result.clone()));
        Ok(checksum_result)
    }

    fn initialize_checksum_state(&self, config: &ChecksumConfig) -> ChecksumState {
        match config.algorithm {
            ChecksumAlgorithm::CRC8 => ChecksumState::CRC8(0),
            ChecksumAlgorithm::CRC16 => ChecksumState::CRC16(0),
            ChecksumAlgorithm::CRC32 => ChecksumState::CRC32(0),
            ChecksumAlgorithm::CRC64 => ChecksumState::CRC64(0),
            ChecksumAlgorithm::MD5 => ChecksumState::MD5(md5::Context::new()),
            ChecksumAlgorithm::SHA1 => ChecksumState::SHA1(sha1::Sha1::new()),
            ChecksumAlgorithm::SHA256 => ChecksumState::SHA256(sha2::Sha256::new()),
            ChecksumAlgorithm::SHA512 => ChecksumState::SHA512(sha2::Sha512::new()),
            ChecksumAlgorithm::Adler32 => ChecksumState::Adler32(1),
            ChecksumAlgorithm::Fletcher16 => ChecksumState::Fletcher16(0xFFFF, 0),
            ChecksumAlgorithm::Fletcher32 => ChecksumState::Fletcher32(0, 0),
            ChecksumAlgorithm::Fletcher64 => ChecksumState::Fletcher64(0, 0, 0, 0),
            ChecksumAlgorithm::Custom(_) => ChecksumState::Custom(0),
        }
    }

    fn update_checksum(&self, state: &mut ChecksumState, data: &[u8], config: &ChecksumConfig) {
        match state {
            ChecksumState::CRC8(ref mut crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0x07);
                for &byte in data {
                    *crc = self.crc8_update(*crc, byte, polynomial);
                }
            },
            ChecksumState::CRC16(ref mut crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0x1021);
                for &byte in data {
                    *crc = self.crc16_update(*crc, byte, polynomial);
                }
            },
            ChecksumState::CRC32(ref mut crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0xEDB88320);
                for &byte in data {
                    *crc = self.crc32_update(*crc, byte, polynomial);
                }
            },
            ChecksumState::CRC64(ref mut crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0xC96C5795D7870F42);
                for &byte in data {
                    *crc = self.crc64_update(*crc, byte, polynomial);
                }
            },
            ChecksumState::MD5(ref mut context) => {
                use md5::Digest;
                context.consume(data);
            },
            ChecksumState::SHA1(ref mut context) => {
                use sha1::Digest;
                context.update(data);
            },
            ChecksumState::SHA256(ref mut context) => {
                use sha2::Digest;
                context.update(data);
            },
            ChecksumState::SHA512(ref mut context) => {
                use sha2::Digest;
                context.update(data);
            },
            ChecksumState::Adler32(ref mut adler) => {
                for &byte in data {
                    *adler = self.adler32_update(*adler, byte);
                }
            },
            ChecksumState::Fletcher16(ref mut sum1, ref mut sum2) => {
                for &byte in data {
                    *sum1 = (*sum1 + byte as u16) % 255;
                    *sum2 = (*sum2 + *sum1) % 255;
                }
            },
            ChecksumState::Fletcher32(ref mut sum1, ref mut sum2) => {
                for &byte in data {
                    *sum1 = (*sum1 + byte as u32) % 65535;
                    *sum2 = (*sum2 + *sum1) % 65535;
                }
            },
            ChecksumState::Fletcher64(ref mut sum1, ref mut sum2, ref mut sum3, ref mut sum4) => {
                for &byte in data {
                    let b = byte as u64;
                    *sum1 = (*sum1 + b) % 4294967295;
                    *sum2 = (*sum2 + *sum1) % 4294967295;
                    *sum3 = (*sum3 + *sum2) % 4294967295;
                    *sum4 = (*sum4 + *sum3) % 4294967295;
                }
            },
            ChecksumState::Custom(ref mut state) => {
                *state += data.len() as u64;
            },
        }
    }

    fn finalize_checksum(&self, state: ChecksumState, config: &ChecksumConfig) -> String {
        let checksum = match state {
            ChecksumState::CRC8(crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0x07);
                self.crc8_finalize(*crc, polynomial)
            },
            ChecksumState::CRC16(crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0x1021);
                self.crc16_finalize(*crc, polynomial)
            },
            ChecksumState::CRC32(crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0xEDB88320);
                self.crc32_finalize(*crc, polynomial)
            },
            ChecksumState::CRC64(crc) => {
                let polynomial = config.custom_polynomial.unwrap_or(0xC96C5795D7870F42);
                self.crc64_finalize(*crc, polynomial)
            },
            ChecksumState::MD5(context) => {
                use md5::Digest;
                format!("{:x}", context.compute())
            },
            ChecksumState::SHA1(context) => {
                use sha1::Digest;
                format!("{:x}", context.compute())
            },
            ChecksumState::SHA256(context) => {
                use sha2::Digest;
                format!("{:x}", context.compute())
            },
            ChecksumState::SHA512(context) => {
                use sha2::Digest;
                format!("{:x}", context.compute())
            },
            ChecksumState::Adler32(adler) => format!("{:08x}", *adler),
            ChecksumState::Fletcher16(sum1, sum2) => format!("{:04x}{:04x}", *sum2, *sum1),
            ChecksumState::Fletcher32(sum1, sum2) => format!("{:08x}{:08x}", *sum2, *sum1),
            ChecksumState::Fletcher64(sum1, sum2, sum3, sum4) => {
                format!("{:08x}{:08x}{:08x}{:08x}", *sum4, *sum3, *sum2, *sum1)
            },
            ChecksumState::Custom(state) => format!("{:x}", *state),
        };

        if config.use_lowercase {
            checksum.to_lowercase()
        } else {
            checksum
        }
    }

    fn crc8_update(&self, crc: u8, data: u8, polynomial: u8) -> u8 {
        crc ^ data
    }

    fn crc8_finalize(&self, crc: u8, polynomial: u8) -> String {
        format!("{:02x}", crc)
    }

    fn crc16_update(&self, crc: u16, data: u8, polynomial: u16) -> u16 {
        let mut crc = crc;
        crc ^= (data as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
        crc
    }

    fn crc16_finalize(&self, crc: u16, polynomial: u16) -> String {
        format!("{:04x}", crc)
    }

    fn crc32_update(&self, crc: u32, data: u8, polynomial: u32) -> u32 {
        let mut crc = crc;
        crc ^= data as u32;
        for _ in 0..8 {
            if crc & 0x80000000 != 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
        crc
    }

    fn crc32_finalize(&self, crc: u32, polynomial: u32) -> String {
        format!("{:08x}", crc)
    }

    fn crc64_update(&self, crc: u64, data: u8, polynomial: u64) -> u64 {
        let mut crc = crc;
        crc ^= data as u64;
        for _ in 0..8 {
            if crc & 0x80000000000000000 != 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
        crc
    }

    fn crc64_finalize(&self, crc: u64, polynomial: u64) -> String {
        format!("{:016x}", crc)
    }

    fn adler32_update(&self, adler: u32, data: u8) -> u32 {
        let mut a = adler & 0xFFFF;
        let mut b = (adler >> 16) & 0xFFFF;
        
        a = (a + data as u32) % 65521;
        b = (b + a) % 65521;
        
        (b << 16) | a
    }

    fn calculate_crc8(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let polynomial = config.custom_polynomial.unwrap_or(0x07);
        let mut crc = config.custom_initial_value.unwrap_or(0) as u8;
        
        for &byte in data {
            crc = self.crc8_update(crc, byte, polynomial);
        }
        
        Ok(self.crc8_finalize(crc, polynomial))
    }

    fn calculate_crc16(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let polynomial = config.custom_polynomial.unwrap_or(0x1021);
        let mut crc = config.custom_initial_value.unwrap_or(0) as u16;
        
        for &byte in data {
            crc = self.crc16_update(crc, byte, polynomial);
        }
        
        Ok(self.crc16_finalize(crc, polynomial))
    }

    fn calculate_crc32(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let polynomial = config.custom_polynomial.unwrap_or(0xEDB88320);
        let mut crc = config.custom_initial_value.unwrap_or(0) as u32;
        
        for &byte in data {
            crc = self.crc32_update(crc, byte, polynomial);
        }
        
        Ok(self.crc32_finalize(crc, polynomial))
    }

    fn calculate_crc64(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let polynomial = config.custom_polynomial.unwrap_or(0xC96C5795D7870F42);
        let mut crc = config.custom_initial_value.unwrap_or(0) as u64;
        
        for &byte in data {
            crc = self.crc64_update(crc, byte, polynomial);
        }
        
        Ok(self.crc64_finalize(crc, polynomial))
    }

    fn calculate_md5(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        use md5::Digest;
        let mut hasher = md5::Context::new();
        hasher.consume(data);
        Ok(format!("{:x}", hasher.compute()))
    }

    fn calculate_sha1(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        use sha1::Digest;
        let mut hasher = sha1::Sha1::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.compute()))
    }

    fn calculate_sha256(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.compute()))
    }

    fn calculate_sha512(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::Digest;
        let mut hasher = sha2::Sha512::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.compute()))
    }

    fn calculate_adler32(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let mut adler = config.custom_initial_value.unwrap_or(1);
        
        for &byte in data {
            adler = self.adler32_update(adler, byte);
        }
        
        Ok(format!("{:08x}", adler))
    }

    fn calculate_fletcher16(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let mut sum1 = config.custom_initial_value.unwrap_or(0) as u16;
        let mut sum2 = 0u16;
        
        for &byte in data {
            sum1 = (sum1 + byte as u16) % 255;
            sum2 = (sum2 + sum1) % 255;
        }
        
        Ok(format!("{:04x}{:04x}", sum2, sum1))
    }

    fn calculate_fletcher32(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let mut sum1 = config.custom_initial_value.unwrap_or(0) as u32;
        let mut sum2 = 0u32;
        
        for &byte in data {
            sum1 = (sum1 + byte as u32) % 65535;
            sum2 = (sum2 + sum1) % 65535;
        }
        
        Ok(format!("{:08x}{:08x}", sum2, sum1))
    }

    fn calculate_fletcher64(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let mut sum1 = config.custom_initial_value.unwrap_or(0) as u64;
        let mut sum2 = 0u64;
        let mut sum3 = 0u64;
        let mut sum4 = 0u64;
        
        for &byte in data {
            let b = byte as u64;
            sum1 = (sum1 + b) % 4294967295;
            sum2 = (sum2 + sum1) % 4294967295;
            sum3 = (sum3 + sum2) % 4294967295;
            sum4 = (sum4 + sum3) % 4294967295;
        }
        
        Ok(format!("{:08x}{:08x}{:08x}{:08x}", sum4, sum3, sum2, sum1))
    }

    fn calculate_custom(&self, data: &[u8], config: &ChecksumConfig) -> Result<String, Box<dyn std::error::Error>> {
        let mut sum = config.custom_initial_value.unwrap_or(0);
        
        for &byte in data {
            sum += byte as u64;
        }
        
        Ok(format!("{:x}", sum))
    }

    pub fn set_algorithm(&self, algorithm: ChecksumAlgorithm) {
        let mut current_algorithm = self.algorithm.write();
        *current_algorithm = algorithm;
    }

    pub fn get_algorithm(&self) -> ChecksumAlgorithm {
        self.algorithm.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<ChecksumEvent> {
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

    pub fn get_supported_algorithms(&self) -> Vec<ChecksumAlgorithm> {
        vec![
            ChecksumAlgorithm::CRC8,
            ChecksumAlgorithm::CRC16,
            ChecksumAlgorithm::CRC32,
            ChecksumAlgorithm::CRC64,
            ChecksumAlgorithm::MD5,
            ChecksumAlgorithm::SHA1,
            ChecksumAlgorithm::SHA256,
            ChecksumAlgorithm::SHA512,
            ChecksumAlgorithm::Adler32,
            ChecksumAlgorithm::Fletcher16,
            ChecksumAlgorithm::Fletcher32,
            ChecksumAlgorithm::Fletcher64,
        ]
    }

    pub fn can_calculate_algorithm(&self, algorithm: &ChecksumAlgorithm) -> bool {
        self.get_supported_algorithms().contains(algorithm)
    }

    pub fn verify_checksum(&self, data: &[u8], expected_checksum: &str, config: ChecksumConfig) -> Result<bool, Box<dyn std::error::Error>> {
        let calculated = self.calculate(data, config)?;
        Ok(calculated.hex_checksum == expected_checksum)
    }

    pub fn verify_file_checksum(&self, file_path: &str, expected_checksum: &str, config: ChecksumConfig) -> Result<bool, Box<dyn std::error::Error>> {
        let calculated = self.calculate_file(file_path, config)?;
        Ok(calculated.hex_checksum == expected_checksum)
    }

    pub fn clone_calculator(&self) -> ChecksumCalculator {
        let mut new_calculator = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_algorithm(),
        );

        new_calculator
    }

    pub fn reset(&self) {
        let _ = self.event_sender.send(ChecksumEvent::Error("Calculator reset".to_string()));
    }
}

#[derive(Debug, Clone)]
enum ChecksumState {
    CRC8(u8),
    CRC16(u16),
    CRC32(u32),
    CRC64(u64),
    MD5(md5::Context),
    SHA1(sha1::Sha1),
    SHA256(sha2::Sha256),
    SHA512(sha2::Sha512),
    Adler32(u32),
    Fletcher16(u16, u16),
    Fletcher32(u32, u32),
    Fletcher64(u64, u64, u64, u64),
    Custom(u64),
}

impl Default for ChecksumCalculator {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Checksum Calculator".to_string(),
            ChecksumAlgorithm::SHA256,
        )
    }
}

impl Default for ChecksumAlgorithm {
    fn default() -> Self {
        ChecksumAlgorithm::SHA256
    }
}

impl Default for ChecksumEvent {
    fn default() -> Self {
        ChecksumEvent::CalculationStarted
    }
}

impl Default for ChecksumResult {
    fn default() -> Self {
        Self {
            algorithm: "SHA256".to_string(),
            checksum: String::new(),
            hex_checksum: String::new(),
            bytes_processed: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for ChecksumConfig {
    fn default() -> Self {
        Self {
            algorithm: ChecksumAlgorithm::SHA256,
            chunk_size: 4096,
            use_lowercase: false,
            include_prefix: false,
            custom_polynomial: None,
            custom_initial_value: None,
        }
    }
}
