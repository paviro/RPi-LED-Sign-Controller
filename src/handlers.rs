use axum::{
    extract::{State, Path, Request},
    response::{Json, Html, IntoResponse},
    http::{header, StatusCode},
};
use std::{sync::Arc, time::{Duration, Instant}};
use tokio::sync::Mutex;
use log::{info, error, debug, warn};
use crate::static_assets::StaticAssets;
use mime_guess;
use serde::{Serialize, Deserialize};

use crate::{
    display_manager::DisplayManager,
    models::{DisplayContent, BrightnessSettings, ReorderRequest},
    app_storage::SharedStorage,
};

// Type alias for our application state
type AppState = (Arc<Mutex<DisplayManager>>, SharedStorage);

// Handler for getting all playlist items
pub async fn get_playlist_items(
    State((display, _)): State<AppState>,
) -> Json<Vec<DisplayContent>> {
    debug!("Getting all playlist items");
    let display = display.lock().await;
    Json(display.playlist.items.clone())
}

// Handler for creating a new playlist item
pub async fn create_playlist_item(
    State((display, storage)): State<AppState>,
    Json(item): Json<DisplayContent>,
) -> (StatusCode, Json<DisplayContent>) {
    debug!("Creating new playlist item");
    
    // No need to check for empty ID - deserialization already handled it
    
    let mut display_guard = display.lock().await;
    display_guard.playlist.items.push(item.clone());
    
    // Save updated playlist
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_playlist(&display_guard.playlist) {
        error!("Failed to save playlist after adding new item");
    }
    
    (StatusCode::CREATED, Json(item))
}

// Handler for getting a specific playlist item
pub async fn get_playlist_item(
    State((display, _)): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DisplayContent>, StatusCode> {
    debug!("Getting playlist item with ID: {}", id);
    
    let display_guard = display.lock().await;
    
    // Find the item with the given ID
    if let Some(item) = display_guard.playlist.items.iter().find(|item| item.id == id) {
        Ok(Json(item.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for updating a specific playlist item
pub async fn update_playlist_item(
    State((display, storage)): State<AppState>,
    Path(id): Path<String>,
    Json(updated_item): Json<DisplayContent>,
) -> Result<StatusCode, StatusCode> {
    debug!("Updating playlist item with ID: {}", id);
    
    let mut display_guard = display.lock().await;
    
    // Find the index of the item with the given ID
    if let Some(index) = display_guard.playlist.items.iter().position(|item| item.id == id) {
        // Make sure we keep the original ID
        let mut item_to_update = updated_item;
        item_to_update.id = id;
        
        // Update the item
        display_guard.playlist.items[index] = item_to_update;
        
        // Save updated playlist
        let storage_guard = storage.lock().unwrap();
        if !storage_guard.save_playlist(&display_guard.playlist) {
            error!("Failed to save playlist after updating item");
        }
        
        // Reset display state if currently showing this item
        if display_guard.playlist.active_index == index {
            display_guard.reset_display_state();
        }
        
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for deleting a specific playlist item
pub async fn delete_playlist_item(
    State((display, storage)): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting playlist item with ID: {}", id);
    
    let mut display_guard = display.lock().await;
    
    // Find the index of the item with the given ID
    if let Some(index) = display_guard.playlist.items.iter().position(|item| item.id == id) {
        // Remove the item
        display_guard.playlist.items.remove(index);
        
        // Adjust active_index if necessary
        if !display_guard.playlist.items.is_empty() {
            if display_guard.playlist.active_index >= index {
                display_guard.playlist.active_index = 
                    display_guard.playlist.active_index.saturating_sub(1);
            }
        } else {
            display_guard.playlist.active_index = 0;
        }
        
        // Save updated playlist
        let storage_guard = storage.lock().unwrap();
        if !storage_guard.save_playlist(&display_guard.playlist) {
            error!("Failed to save playlist after deleting item");
        }
        
        // Reset display state
        display_guard.reset_display_state();
        
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for reordering playlist items
pub async fn reorder_playlist_items(
    State((display, storage)): State<AppState>,
    Json(reorder_request): Json<ReorderRequest>,
) -> Result<StatusCode, StatusCode> {
    debug!("Reordering playlist items");
    
    let mut display_guard = display.lock().await;
    
    // Check if all requested IDs exist in the playlist
    for id in &reorder_request.item_ids {
        if !display_guard.playlist.items.iter().any(|item| &item.id == id) {
            warn!("Reorder request contains unknown item ID: {}", id);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Check if the request contains all items
    if reorder_request.item_ids.len() != display_guard.playlist.items.len() {
        warn!("Reorder request doesn't match existing items count: {} vs {}", 
              reorder_request.item_ids.len(), display_guard.playlist.items.len());
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Create a new ordered list based on the requested order
    let mut new_items: Vec<DisplayContent> = Vec::with_capacity(display_guard.playlist.items.len());
    
    for id in &reorder_request.item_ids {
        if let Some(item) = display_guard.playlist.items.iter().find(|item| &item.id == id).cloned() {
            new_items.push(item);
        }
    }
    
    // Replace the items with the new ordered list
    display_guard.playlist.items = new_items;
    
    // Reset display state
    display_guard.reset_display_state();
    
    // Save updated playlist
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_playlist(&display_guard.playlist) {
        error!("Failed to save playlist after reordering items");
    }
    
    Ok(StatusCode::OK)
}

// Add this helper method to DisplayManager
impl DisplayManager {
    pub fn reset_display_state(&mut self) {
        // Reset the display state to start fresh with current item
        self.last_transition = Instant::now();
        self.current_repeat = 0;
        self.completed_scrolls = 0;
        self.scroll_position = self.display_width;
    }
}

// Background task to handle display updates
pub async fn display_loop(display: Arc<Mutex<DisplayManager>>) {
    info!("Starting display update loop");
    let mut accumulated_time: f32 = 0.0;
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut last_stats_time = Instant::now();
    
    // Preview timeout in seconds
    const PREVIEW_TIMEOUT: u64 = 5;
    
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        
        let mut display_guard = display.lock().await;
        
        // Check for preview mode timeout
        display_guard.check_preview_timeout(PREVIEW_TIMEOUT);
        
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
        
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}

// Update index handler to use embedded assets and properly handle ownership
pub async fn index_handler() -> Html<String> {
    let index_html = StaticAssets::get("index.html")
        .expect("index.html not found in embedded assets");
    
    // Convert the content to string
    let content = std::str::from_utf8(index_html.data.as_ref())
        .expect("Failed to convert index.html to UTF-8")
        .to_string();
    
    // Process the content to fix paths if needed
    // Note: This is a basic approach - you might need more sophisticated HTML parsing
    let processed_content = content;
    
    Html(processed_content)
}

// New handler to get the current brightness
pub async fn get_brightness(
    State((display, _)): State<AppState>,
) -> Json<BrightnessSettings> {
    info!("Getting current brightness");
    let display = display.lock().await;
    
    let brightness = display.get_brightness();
    
    Json(BrightnessSettings {
        brightness
    })
}

// Handler for updating brightness - applies brightness through color scaling
pub async fn update_brightness(
    State((display, storage)): State<AppState>,
    Json(settings): Json<BrightnessSettings>,
) -> StatusCode {
    debug!("Updating brightness to {}", settings.brightness);
    
    let mut display = display.lock().await;
    
    // Apply brightness scaling through DisplayManager's method
    display.set_brightness(settings.brightness);
    
    // Save the brightness setting for persistence across restarts
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_brightness(settings.brightness) {
        error!("Failed to save brightness setting");
    }
    
    StatusCode::OK
}

// Add a function to serve files from the _next directory
pub async fn next_assets_handler(req: Request) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches("/_next");
    let full_path = format!("_next{}", path);
    
    debug!("Serving next asset: {}", full_path);
    
    // Try to get the file from the embedded assets
    match StaticAssets::get(&full_path) {
        Some(content) => {
            // Get MIME type
            let content_type = mime_guess::from_path(&full_path)
                .first_or_octet_stream()
                .to_string();
            
            // Return file with appropriate content type
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content.data
            ).into_response()
        },
        None => {
            warn!("Next asset not found: {}", full_path);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

// Similar to next_assets_handler
pub async fn static_assets_handler(Path(path): Path<String>) -> impl IntoResponse {
    let full_path = format!("static/{}", path);
    
    debug!("Serving static asset: {}", full_path);
    
    match StaticAssets::get(&path) {
        Some(content) => {
            // Get MIME type
            let content_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content.data
            ).into_response()
        },
        None => {
            warn!("Static asset not found: {}", path);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

// New struct for preview mode state
#[derive(Serialize, Deserialize)]
pub struct PreviewModeState {
    pub active: bool,
}

// Handler for starting preview mode with a content item
pub async fn start_preview_mode(
    State((display, _)): State<AppState>,
    Json(preview_item): Json<DisplayContent>,
) -> StatusCode {
    // Display manager handles logging based on state changes
    let mut display_guard = display.lock().await;
    display_guard.enter_preview_mode(preview_item);
    StatusCode::OK
}

// Handler for exiting preview mode
pub async fn exit_preview_mode(
    State((display, _)): State<AppState>,
) -> StatusCode {
    // Display manager handles logging based on state changes
    let mut display_guard = display.lock().await;
    display_guard.exit_preview_mode();
    StatusCode::OK
}

// Handler for checking preview mode status
pub async fn get_preview_mode_status(
    State((display, _)): State<AppState>,
) -> Json<PreviewModeState> {
    let display_guard = display.lock().await;
    let active = display_guard.is_in_preview_mode();
    Json(PreviewModeState { active })
}

// Handler for pinging preview mode to keep it active
pub async fn ping_preview_mode(
    State((display, _)): State<AppState>,
) -> StatusCode {
    let mut display_guard = display.lock().await;
    if display_guard.ping_preview_mode() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
} 