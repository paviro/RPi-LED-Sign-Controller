mod display_manager;
mod handlers;
mod models;
mod static_assets;
mod app_storage;
mod storage_manager;
mod led_driver;
mod embedded_graphics_support;
mod config;
mod privilege;

use crate::app_storage::{create_storage};
use crate::led_driver::create_driver;
use crate::privilege::{check_root_privileges, drop_privileges};

use axum::{
    routing::{post, get, put, delete},
    Router,
};
use display_manager::DisplayManager;
use handlers::{index_handler, editor_handler, display_loop, get_brightness, update_brightness, get_playlist_items, create_playlist_item, get_playlist_item, update_playlist_item, delete_playlist_item, reorder_playlist_items};
use std::{sync::Arc, net::SocketAddr};
use tokio::sync::Mutex;
use log::{info, warn, error, debug, LevelFilter};
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
                log::Level::Trace => record.level().to_string().purple(),
            };

            // Apply appropriate colors to the message based on level
            let message = match record.level() {
                log::Level::Error => record.args().to_string().red(),
                log::Level::Warn => record.args().to_string().yellow(),
                log::Level::Info => record.args().to_string().normal(),
                log::Level::Debug => record.args().to_string().blue(),
                log::Level::Trace => record.args().to_string().purple(),
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
    
    // Check for root privileges before doing anything else
    if let Err(e) = check_root_privileges() {
        error!("{}", e);
        std::process::exit(1);
    }
    
    // Set higher priority for the process if possible
    #[cfg(target_os = "linux")]
    unsafe {
        // Set nice level to -20
        libc::nice(-20);
        debug!("Set process priority to -20");
        
        // Set real-time scheduling with high priority
        let pid = libc::getpid();
        let sched_param = libc::sched_param { sched_priority: 99 };
        if libc::sched_setscheduler(pid, libc::SCHED_FIFO, &sched_param) != 0 {
            let err = std::io::Error::last_os_error();
            warn!("Failed to set real-time scheduling: {}", err);
        } else {
            debug!("Set real-time scheduling policy with priority 99");
        }
    }

    // Initialize configuration
    let display_config = init_config();
    
    // Validate configuration
    if let Err(errors) = display_config.validate() {
        for error in errors {
            error!("{}", error);
        }
        std::process::exit(1);
    }
    
    // After configuration validation, but before driver initialization
    let storage = create_storage(None);

    // Create the driver - this might drop privileges
    info!("Initializing LED matrix driver (requires elevated privileges)");
    let driver = match create_driver(&display_config) {
        Ok(driver) => driver,
        Err(e) => {
            error!("Failed to initialize LED matrix driver: {}", e);
            std::process::exit(1);
        }
    };

    // Now drop privileges explicitly if the driver didn't do it
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = drop_privileges() {
            error!("Failed to drop privileges: {}", e);
        }
    }
    
    // Initialize display manager with the pre-created driver
    let display = {
        let storage_guard = storage.lock().unwrap();
        let persisted_playlist = storage_guard.load_playlist();
        let persisted_brightness = storage_guard.load_brightness();
        
        let mut display_manager = if let Some(playlist) = persisted_playlist {
            info!("Loaded playlist from filesystem with {} items", playlist.items.len());
            DisplayManager::with_playlist_config_and_driver(playlist, &display_config, driver)
        } else {
            info!("No saved playlist found, using default");
            DisplayManager::with_config_and_driver(&display_config, driver)
        };
        
        // Apply the saved brightness if available
        if let Some(brightness) = persisted_brightness {
            info!("Applying saved brightness: {}", brightness);
            display_manager.set_brightness(brightness);
        }
        
        Arc::new(Mutex::new(display_manager))
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
        // New RESTful playlist endpoints
        .route("/api/playlist/items", get(get_playlist_items))
        .route("/api/playlist/items", post(create_playlist_item))
        .route("/api/playlist/items/:id", get(get_playlist_item))
        .route("/api/playlist/items/:id", put(update_playlist_item))
        .route("/api/playlist/items/:id", delete(delete_playlist_item))
        .route("/api/playlist/reorder", put(reorder_playlist_items))
        
        // Settings endpoints
        .route("/api/settings/brightness", get(get_brightness))
        .route("/api/settings/brightness", put(update_brightness))
        
        .with_state((display.clone(), storage));
    
    // Simplified static assets setup
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/editor", get(editor_handler))
        .route("/_next/*path", get(handlers::next_assets_handler))
        .route("/static/*path", get(handlers::static_assets_handler))
        .nest("", api_routes);
    
    let ip_addr = display_config.interface.parse::<std::net::IpAddr>()
        .expect("Invalid network interface address");

    let addr = SocketAddr::from((ip_addr, display_config.port));

    info!("Server running on http://{}", addr);
    
    if let Err(e) = axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to bind to address {}: {}", addr, e);
                std::process::exit(1);
            }),
        app,
    ).await {
        error!("Server error: {}", e);
    }

    info!("Application exiting, cleaning up display...");
    let mut display_guard = display.lock().await;
    display_guard.shutdown();
}
