mod display_manager;
mod handlers;
mod models;
mod static_assets;
mod playlist_storage;
mod storage_manager;
mod led_driver;
mod embedded_graphics_support;
mod config;

use axum::{
    routing::{post, get},
    Router,
};
use axum_embed::ServeEmbed;
use static_assets::StaticAssets;
use display_manager::DisplayManager;
use handlers::{update_playlist, get_playlist, index_handler, editor_handler, display_loop, get_brightness, update_brightness};
#[allow(unused_imports)]
use playlist_storage::{create_storage, SharedStorage};
use std::{sync::Arc, net::SocketAddr};
use tokio::sync::Mutex;
use log::{info, error, debug, LevelFilter};
use env_logger::Builder;
use chrono::Local;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use config::init_config;
use colored::*;

// Global shutdown flag
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    // Initialize the logger with a custom format that includes timestamps and colors
    Builder::new()
        .format(|buf, record| {
            // Color based on log level
            let level = match record.level() {
                log::Level::Error => record.level().to_string().red().bold(),
                log::Level::Warn => record.level().to_string().yellow().bold(),
                log::Level::Info => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().blue(),
                log::Level::Trace => record.level().to_string().white(),
            };

            // Apply appropriate colors to the message based on level
            let message = match record.level() {
                log::Level::Error => record.args().to_string().red(),
                log::Level::Warn => record.args().to_string().yellow(),
                log::Level::Info => record.args().to_string().green(),
                log::Level::Debug => record.args().to_string().normal(),
                log::Level::Trace => record.args().to_string().normal(),
            };

            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                level,
                message
            )
        })
        .filter(None, LevelFilter::Info) // Set default log level to Info
        .parse_env("RUST_LOG") // Allow overriding with RUST_LOG environment variable
        .init();
    
    info!("Starting LED Sign Controller");
    
    // Set higher priority for the process if possible
    #[cfg(target_os = "linux")]
    unsafe {
        libc::nice(-20);
        debug!("Set process priority to -20");
    }

    // Create storage with default path in home directory
    let storage = create_storage(None);
    debug!("Storage initialized");
    
    // Initialize configuration
    let display_config = init_config();
    
    // Validate configuration
    if let Err(errors) = display_config.validate() {
        for error in errors {
            error!("{}", error);
        }
        std::process::exit(1);
    }
    
    // Initialize display manager with persisted playlist if available
    let display = {
        let storage_guard = storage.lock().unwrap();
        let persisted_playlist = storage_guard.load_playlist();
        
        if let Some(playlist) = persisted_playlist {
            info!("Loaded playlist from filesystem with {} items", playlist.items.len());
            Arc::new(Mutex::new(DisplayManager::with_playlist_and_config(playlist, display_config)))
        } else {
            info!("No saved playlist found, using default");
            Arc::new(Mutex::new(DisplayManager::with_config(display_config)))
        }
    };
    
    // Set up signal handlers for clean shutdown
    let display_for_shutdown = display.clone();
    if let Err(e) = ctrlc::set_handler(move || {
        info!("Received termination signal, shutting down...");
        SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
        
        // Try to get a lock on the display and shut it down
        // Using try_lock to avoid deadlocks since we're in a signal handler
        if let Ok(mut display_guard) = display_for_shutdown.try_lock() {
            // Clear the display before shutting down
            display_guard.shutdown();
        } else {
            println!("Could not acquire display lock for shutdown - display might not be properly cleared");
        }
        
        std::process::exit(0);
    }) {
        error!("Error setting Ctrl-C handler: {}", e);
    }
    
    // Spawn display update task
    let display_clone = display.clone();
    tokio::spawn(async move {
        debug!("Display update task started");
        display_loop(display_clone).await;
    });
    
    // API routes with shared storage
    let api_routes = Router::new()
        .route("/playlist", post(update_playlist))
        .route("/playlist", get(get_playlist))
        .route("/brightness", get(get_brightness))
        .route("/brightness", post(update_brightness))
        .with_state((display.clone(), storage));
    
    // Static asset handler
    let static_assets = ServeEmbed::<StaticAssets>::new();
    
    // Main router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/editor", get(editor_handler))
        .nest("", api_routes)
        .nest_service("/static", static_assets);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server running on http://{}", addr);
    
    if let Err(e) = axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .unwrap(),
        app,
    ).await {
        error!("Server error: {}", e);
    }

    info!("Application exiting, cleaning up display...");
    let mut display_guard = display.lock().await;
    display_guard.shutdown();
}