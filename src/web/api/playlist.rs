use crate::models::playlist::PlayListItem;
use crate::models::settings::ReorderRequest;
use crate::web::api::CombinedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use log::{debug, error, warn};
use crate::web::api::events::PlaylistAction;

// Handler for getting all playlist items
pub async fn get_playlist_items(
    State(combined_state): State<CombinedState>,
) -> Json<Vec<PlayListItem>> {
    debug!("Getting all playlist items");
    let ((display, _), _) = combined_state;
    let display = display.lock().await;
    Json(display.playlist.items.clone())
}

// Handler for creating a new playlist item
pub async fn create_playlist_item(
    State(combined_state): State<CombinedState>,
    Json(item): Json<PlayListItem>,
) -> (StatusCode, Json<PlayListItem>) {
    debug!("Creating new playlist item");
    
    // No need to check for empty ID - deserialization already handled it
    let ((display, storage), event_state) = combined_state;
    let mut display_guard = display.lock().await;
    display_guard.playlist.items.push(item.clone());
    
    // Save updated playlist
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_playlist(&display_guard.playlist) {
        error!("Failed to save playlist after adding new item");
    }
    
    // Broadcast the playlist update
    let event_state_guard = event_state.lock().unwrap();
    event_state_guard.broadcast_playlist_update(
        display_guard.playlist.items.clone(),
        PlaylistAction::Add
    );
    
    (StatusCode::CREATED, Json(item))
}

// Handler for getting a specific playlist item
pub async fn get_playlist_item(
    State(combined_state): State<CombinedState>,
    Path(id): Path<String>,
) -> Result<Json<PlayListItem>, StatusCode> {
    debug!("Getting playlist item with ID: {}", id);
    
    let ((display, _), _) = combined_state;
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
    State(combined_state): State<CombinedState>,
    Path(id): Path<String>,
    Json(updated_item): Json<PlayListItem>,
) -> Result<Json<PlayListItem>, StatusCode> {
    debug!("Updating playlist item with ID: {}", id);
    
    let ((display, storage), event_state) = combined_state;
    let mut display_guard = display.lock().await;
    
    if let Some(index) = display_guard.playlist.items.iter().position(|item| item.id == id) {
        let mut item_to_update = updated_item;
        item_to_update.id = id;
        
        display_guard.playlist.items[index] = item_to_update.clone();
        
        // Save updated playlist
        let storage_guard = storage.lock().unwrap();
        if !storage_guard.save_playlist(&display_guard.playlist) {
            error!("Failed to save playlist after updating item");
        }
        
        // Broadcast the playlist update
        let event_state_guard = event_state.lock().unwrap();
        event_state_guard.broadcast_playlist_update(
            display_guard.playlist.items.clone(),
            PlaylistAction::Update
        );
        
        // Reset display state if currently showing this item
        if display_guard.playlist.active_index == index {
            display_guard.reset_display_state();
        }
        
        Ok(Json(item_to_update))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for deleting a specific playlist item
pub async fn delete_playlist_item(
    State(combined_state): State<CombinedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting playlist item with ID: {}", id);
    
    let ((display, storage), event_state) = combined_state;
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
        
        // Broadcast the playlist update
        let event_state_guard = event_state.lock().unwrap();
        event_state_guard.broadcast_playlist_update(
            display_guard.playlist.items.clone(),
            PlaylistAction::Delete
        );
        
        // Reset display state
        display_guard.reset_display_state();
        
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for reordering playlist items
pub async fn reorder_playlist_items(
    State(combined_state): State<CombinedState>,
    Json(reorder_request): Json<ReorderRequest>,
) -> Result<Json<Vec<PlayListItem>>, StatusCode> {
    debug!("Reordering playlist items");
    
    let ((display, storage), event_state) = combined_state;
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
    let mut new_items: Vec<PlayListItem> = Vec::with_capacity(display_guard.playlist.items.len());
    
    for id in &reorder_request.item_ids {
        if let Some(item) = display_guard.playlist.items.iter().find(|item| &item.id == id).cloned() {
            new_items.push(item);
        }
    }
    
    // Replace the items with the new ordered list
    display_guard.playlist.items = new_items.clone();
    
    // Reset display state
    display_guard.reset_display_state();
    
    // Save updated playlist
    let storage_guard = storage.lock().unwrap();
    if !storage_guard.save_playlist(&display_guard.playlist) {
        error!("Failed to save playlist after reordering items");
    }
    
    // Broadcast the playlist update
    let event_state_guard = event_state.lock().unwrap();
    event_state_guard.broadcast_playlist_update(
        new_items.clone(),
        PlaylistAction::Reorder
    );
    
    // Return the reordered items
    Ok(Json(new_items))
}