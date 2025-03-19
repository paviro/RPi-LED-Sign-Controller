mod display_manager;
mod handlers;
mod models;
mod static_assets;
mod playlist_storage;
mod storage_manager;

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

#[tokio::main]
async fn main() {
    // Initialize the logger with a custom format that includes timestamps
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
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
    
    // Initialize display manager with persisted playlist if available
    let display = {
        let storage_guard = storage.lock().unwrap();
        let persisted_playlist = storage_guard.load_playlist();
        
        if let Some(playlist) = persisted_playlist {
            info!("Loaded playlist from filesystem with {} items", playlist.items.len());
            Arc::new(Mutex::new(DisplayManager::with_playlist(playlist)))
        } else {
            info!("No saved playlist found, using default");
            Arc::new(Mutex::new(DisplayManager::new()))
        }
    };
    
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
}