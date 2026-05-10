use tiffiny_app::{app::TiffinyApp, bootstrap::Bootstrap, shutdown::ShutdownHandler};
use tiffiny_utils::logging::init_logging;
use tracing::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let bootstrap = Bootstrap::new().await?;
    
    let mut app = TiffinyApp::new(bootstrap).await?;
    
    let shutdown_handler = ShutdownHandler::new();
    
    let result = tokio::select! {
        app_result = app.run() => {
            app_result
        },
        shutdown_result = shutdown_handler.wait_for_shutdown() => {
            app.shutdown().await?;
            shutdown_result
        }
    };

    if let Err(e) = result {
        error!("{}", e);
        return Err(e.into());
    }

    Ok(())
}
