use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::{broadcast, mpsc, oneshot};

pub struct ShutdownHandler {
    shutdown_tx: broadcast::Sender<ShutdownReason>,
    shutdown_rx: broadcast::Receiver<ShutdownReason>,
    shutdown_handlers: Arc<RwLock<Vec<ShutdownHandlerFn>>>,
    is_shutting_down: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShutdownReason {
    UserRequest,
    Error(String),
    CriticalError,
    Signal(i32),
    Timeout,
    ResourceExhaustion,
    Emergency,
}

type ShutdownHandlerFn = Arc<dyn Fn(ShutdownReason) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;

impl ShutdownHandler {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(10);
        
        Self {
            shutdown_tx,
            shutdown_rx,
            shutdown_handlers: Arc::new(RwLock::new(Vec::new())),
            is_shutting_down: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn wait_for_shutdown(&self) -> ShutdownReason {
        let mut rx = self.shutdown_rx.subscribe();
        
        match rx.recv().await {
            Ok(reason) => reason,
            Err(_) => {
                ShutdownReason::CriticalError
            }
        }
    }

    pub async fn initiate_shutdown(&self, reason: ShutdownReason) -> Result<()> {
        {
            let mut is_shutting_down = self.is_shutting_down.write();
            if *is_shutting_down {
                return Ok(());
            }
            *is_shutting_down = true;
        }


        let _ = self.shutdown_tx.send(reason.clone());

        self.execute_shutdown_handlers(reason.clone()).await?;

        let timeout = std::time::Duration::from_secs(30);
        tokio::time::sleep(timeout).await;

        self.force_cleanup().await?;

        Ok(())
    }

    pub fn register_shutdown_handler<F>(&self, handler: F)
    where
        F: Fn(ShutdownReason) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        let mut handlers = self.shutdown_handlers.write();
        handlers.push(Arc::new(handler));
    }

    pub fn is_shutting_down(&self) -> bool {
        *self.is_shutting_down.read()
    }

    pub fn get_shutdown_receiver(&self) -> broadcast::Receiver<ShutdownReason> {
        self.shutdown_tx.subscribe()
    }

    pub fn create_shutdown_channel(&self) -> (mpsc::Sender<ShutdownReason>, oneshot::Receiver<()>) {
        let (tx, mut rx) = mpsc::channel(10);
        let (done_tx, done_rx) = oneshot::channel();

        let shutdown_rx = self.get_shutdown_receiver();
        let is_shutting_down = self.is_shutting_down.clone();

        tokio::spawn(async move {
            tokio::select! {
                reason = shutdown_rx.recv() => {
                    if let Ok(reason) = reason {
                        let _ = tx.send(reason).await;
                    }
                },
                _ = rx.recv() => {
                }
            }

            let _ = done_tx.send(());
        });

        (tx, done_rx)
    }

    async fn execute_shutdown_handlers(&self, reason: ShutdownReason) -> Result<()> {
        let handlers = self.shutdown_handlers.read();
        
        for (index, handler) in handlers.iter().enumerate() {
            
            match handler(reason.clone()) {
                Ok(()) => {},
                Err(_) => {}
            }
        }

        Ok(())
    }

    async fn force_cleanup(&self) -> Result<()> {

        let cleanup_tasks = vec![
            self.cleanup_temp_files(),
            self.cleanup_memory_pools(),
            self.cleanup_network_connections(),
            self.cleanup_file_handles(),
            self.cleanup_gpu_resources(),
        ];

        let results = futures::future::join_all(cleanup_tasks).await;

        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(()) => {},
                Err(_) => {}
            }
        }

        Ok(())
    }

    async fn cleanup_temp_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dirs = vec![
            std::env::temp_dir(),
            dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
                .join("tiffiny")
                .join("temp"),
        ];

        for temp_dir in temp_dirs {
            if temp_dir.exists() {
                for entry in std::fs::read_dir(&temp_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        let metadata = std::fs::metadata(&path)?;
                        let modified = metadata.modified()?;
                        let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                        
                        if age > std::time::Duration::from_secs(3600) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn cleanup_memory_pools(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        Ok(())
    }

    async fn cleanup_network_connections(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        Ok(())
    }

    async fn cleanup_file_handles(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        Ok(())
    }

    async fn cleanup_gpu_resources(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        Ok(())
    }

    pub async fn graceful_shutdown_with_timeout(&self, timeout: std::time::Duration) -> Result<()> {
        let reason = ShutdownReason::UserRequest;
        
        let (done_tx, done_rx) = oneshot::channel();
        let shutdown_tx = self.shutdown_tx.clone();
        let handlers = self.shutdown_handlers.clone();

        let shutdown_task = tokio::spawn(async move {
            let _ = shutdown_tx.send(reason.clone());

            let handlers = handlers.read();
            for (index, handler) in handlers.iter().enumerate() {
                
                match handler(reason.clone()) {
                    Ok(()) => {},
                    Err(e) => {}
                }
            }

            let _ = done_tx.send(());
        });

        tokio::select! {
            _ = shutdown_task => {
            },
            _ = tokio::time::sleep(timeout) => {
                self.force_cleanup().await?;
            }
        }

        Ok(())
    }

    pub fn setup_signal_handlers(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use signal_hook::{consts::SIGTERM, iterator::Signals};
            
            let shutdown_reasons = Arc::new(std::sync::Mutex::new(vec![
                (signal_hook::consts::SIGINT, ShutdownReason::Signal(2)),
                (signal_hook::consts::SIGTERM, ShutdownReason::Signal(15)),
            ]));

            let mut signals = Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM])?;
            
            let shutdown_tx = self.shutdown_tx.clone();
            let reasons = shutdown_reasons.clone();

            std::thread::spawn(move || {
                for signal in &mut signals {
                    if let Ok(reason) = reasons.lock().unwrap()
                        .iter()
                        .find(|(sig, _)| *sig == signal)
                        .map(|(_, reason)| reason.clone()) {
                        
                        let _ = shutdown_tx.send(reason);
                        break;
                    }
                }
            });
        }

        #[cfg(windows)]
        {
            use winapi::um::winuser::{SetConsoleCtrlHandler, CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT};
            
            let shutdown_tx = self.shutdown_tx.clone();
            
            unsafe {
                SetConsoleCtrlHandler(Some(move |ctrl_type| {
                    let reason = match ctrl_type {
                        CTRL_C_EVENT => ShutdownReason::Signal(2),
                        CTRL_BREAK_EVENT => ShutdownReason::Signal(21),
                        CTRL_CLOSE_EVENT => ShutdownReason::UserRequest,
                        _ => ShutdownReason::Signal(ctrl_type as i32),
                    };
                    
                    let _ = shutdown_tx.send(reason);
                    
                    1
                }), 1);
            }
        }

        Ok(())
    }

    pub fn create_cancellation_token(&self) -> CancellationToken {
        CancellationToken::new(self.is_shutting_down.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
    is_shutting_down: Arc<RwLock<bool>>,
}

impl CancellationToken {
    fn new(is_shutting_down: Arc<RwLock<bool>>) -> Self {
        Self { is_shutting_down }
    }

    pub fn is_cancelled(&self) -> bool {
        *self.is_shutting_down.read()
    }

    pub async fn wait_for_cancellation(&self) {
        while !self.is_cancelled() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    pub fn with_timeout<F, T>(&self, duration: std::time::Duration, future: F) -> impl std::future::Future<Output = Option<T>>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let cancellation_token = self.clone();
        
        async move {
            tokio::select! {
                result = future => Some(result),
                _ = cancellation_token.wait_for_cancellation() => None,
                _ = tokio::time::sleep(duration) => None,
            }
        }
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}
