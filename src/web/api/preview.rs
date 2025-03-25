use crate::models::playlist::PlayListItem;
use crate::web::api::AppState;
use serde::{Deserialize, Serialize};
use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};

// New struct to represent PreviewModeState for API responses
#[derive(Serialize, Deserialize)]
pub struct PreviewModeState {
    pub active: bool,
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
    
    if display_guard.update_preview_ping() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// Handler for starting preview mode with a content item
pub async fn start_preview_mode(
    State((display, _)): State<AppState>,
    Json(preview_item): Json<PlayListItem>,
) -> Json<PlayListItem> {
    let mut display_guard = display.lock().await;
    display_guard.enter_preview_mode(preview_item.clone());
    
    // Return the item that's being previewed
    Json(preview_item)
} 