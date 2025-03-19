use axum::{
    extract::State,
    response::{Json, Html},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::{Duration, Instant}};
use tokio::sync::Mutex;
use log::{info, error, debug};

use crate::{
    display_manager::DisplayManager,
    models::Playlist,
    static_assets::StaticAssets,
    playlist_storage::SharedStorage,
};

#[derive(Serialize, Deserialize)]
pub struct BrightnessSettings {
    pub brightness: u8,
}

// Handler for updating the playlist
pub async fn update_playlist(
    State((display, storage)): State<(Arc<Mutex<DisplayManager>>, SharedStorage)>,
    Json(playlist): Json<Playlist>,
) -> StatusCode {
    debug!("Updating display with playlist containing {} items", playlist.items.len());
    
    let mut display = display.lock().await;
    
    // Check if we need to reinitialize due to brightness change
    let old_brightness = display.get_brightness();
    let new_brightness = playlist.brightness;
    
    if old_brightness != new_brightness {
        debug!("Brightness changed from {} to {}, reinitializing display", 
              old_brightness, new_brightness);
        display.reinitialize_with_brightness(new_brightness);
    }
    
    // Update the display
    display.playlist = playlist.clone();
    display.last_transition = Instant::now();
    display.current_repeat = 0;
    display.completed_scrolls = 0;
    display.scroll_position = display.display_width;
    
    // Save playlist to storage
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_playlist(&playlist) {
        error!("Failed to save playlist to storage");
    }
    
    StatusCode::OK
}

// Background task to handle display updates
pub async fn display_loop(display: Arc<Mutex<DisplayManager>>) {
    info!("Starting display update loop");
    let mut accumulated_time: f32 = 0.0;
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut last_stats_time = Instant::now();
    
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        
        let mut display_guard = display.lock().await;
        
        // Update animation state for border effects
        display_guard.update_animation_state();
        
        // Check if we need to transition to next item
        let transition_occurred = display_guard.check_transition();
        if transition_occurred {
            // Reset happens in check_transition and advance_playlist
            info!("Transitioned to next display item");
        }
        
        let current = display_guard.get_current_content().clone();
        if current.scroll {
            accumulated_time += dt;
            let pixels_to_move = (accumulated_time * current.speed) as i32;
            if pixels_to_move > 0 {
                display_guard.scroll_position -= pixels_to_move;
                accumulated_time = 0.0;
                
                // Reset position when text is off screen
                if display_guard.scroll_position < -display_guard.text_width {
                    display_guard.scroll_position = display_guard.display_width;
                    display_guard.completed_scrolls += 1;  // Increment completed scroll count
                    info!("Completed scroll cycle {} of {}", 
                         display_guard.completed_scrolls, 
                         if current.repeat_count == 0 { "infinite".to_string() } 
                         else { current.repeat_count.to_string() });
                }
            }
        }
        
        // Get the position value first before calling update_display
        let position = display_guard.scroll_position;
        display_guard.update_display(position);
        
        drop(display_guard);
        
        // Log performance stats periodically
        frame_count += 1;
        if now.duration_since(last_stats_time).as_secs() >= 60 {  // Log every minute
            let fps = frame_count as f32 / now.duration_since(last_stats_time).as_secs_f32();
            info!("Display performance: {:.1} FPS", fps);
            frame_count = 0;
            last_stats_time = now;
        }
        
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
}

// Update index handler to use embedded assets and properly handle ownership
pub async fn index_handler() -> Html<String> {
    let index_html = StaticAssets::get("index.html")
        .expect("index.html not found in embedded assets");
    let content = std::str::from_utf8(index_html.data.as_ref())
        .expect("Failed to convert index.html to UTF-8")
        .to_string();  // Convert to owned String
    
    Html(content)
}

// Add this new handler to get the current playlist
pub async fn get_playlist(
    State((display, _)): State<(Arc<Mutex<DisplayManager>>, SharedStorage)>,
) -> Json<Playlist> {
    let display = display.lock().await;
    Json(display.playlist.clone())
}

// Add this handler to serve the editor page
pub async fn editor_handler() -> Html<String> {
    let editor_html = StaticAssets::get("editor.html")
        .expect("editor.html not found in embedded assets");
    let content = std::str::from_utf8(editor_html.data.as_ref())
        .expect("Failed to convert editor.html to UTF-8")
        .to_string();
    
    Html(content)
}

// New handler to get the current brightness
pub async fn get_brightness(
    State((display, storage)): State<(Arc<Mutex<DisplayManager>>, SharedStorage)>,
) -> Json<BrightnessSettings> {
    info!("Getting current brightness");
    let display = display.lock().await;
    
    // First try to get from display manager
    let brightness = display.get_brightness();
    
    // Also check if there's a saved brightness (for logging purposes)
    let storage_guard = storage.lock().unwrap();
    if let Some(saved_brightness) = storage_guard.load_brightness() {
        if saved_brightness != brightness {
            info!("Saved brightness ({}) differs from current brightness ({})", 
                  saved_brightness, brightness);
        }
    }
    
    info!("Current brightness: {}", brightness);
    Json(BrightnessSettings {
        brightness
    })
}

// New handler to update the brightness
pub async fn update_brightness(
    State((display, storage)): State<(Arc<Mutex<DisplayManager>>, SharedStorage)>,
    Json(settings): Json<BrightnessSettings>,
) -> StatusCode {
    // Keep this for server-side logging only
    debug!("Updating brightness to {}", settings.brightness);
    
    let mut display = display.lock().await;
    display.reinitialize_with_brightness(settings.brightness);
    
    // Save the brightness setting separately
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_brightness(settings.brightness) {
        error!("Failed to save brightness setting");
    }
    
    StatusCode::OK
} 