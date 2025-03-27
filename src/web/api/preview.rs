use crate::models::playlist::PlayListItem;
use crate::web::api::CombinedState;
use serde::{Deserialize, Serialize};
use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use crate::utils::uuid::generate_uuid_string;

// New struct to represent PreviewModeState for API responses
#[derive(Serialize, Deserialize)]
pub struct PreviewModeState {
    pub active: bool,
}

// New response type for preview mode operations
#[derive(Serialize, Deserialize)]
pub struct PreviewModeResponse {
    pub item: PlayListItem,
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionCheckRequest {
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionCheckResponse {
    pub is_owner: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StartPreviewRequest {
    pub item: PlayListItem,
}

#[derive(Serialize, Deserialize)]
pub struct PreviewUpdateRequest {
    pub item: PlayListItem,
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExitPreviewRequest {
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PingPreviewRequest {
    pub session_id: String,
}

// Handler for exiting preview mode
pub async fn exit_preview_mode(
    State(combined_state): State<CombinedState>,
    Json(exit_req): Json<ExitPreviewRequest>,
) -> Result<StatusCode, StatusCode> {
    // Display manager handles logging based on state changes
    let ((display, _), event_state) = combined_state;
    let mut display_guard = display.lock().await;
    
    // Check if we're in preview mode
    if !display_guard.is_in_preview_mode() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Verify session ownership
    if !display_guard.is_preview_session_owner(&exit_req.session_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // If we were in preview mode, broadcast an unlock event
    let event_state_guard = event_state.lock().unwrap();
    event_state_guard.broadcast_editor_lock(false, None);
    
    display_guard.exit_preview_mode();
    Ok(StatusCode::OK)
}

// Handler for checking preview mode status
pub async fn get_preview_mode_status(
    State(combined_state): State<CombinedState>,
) -> Json<PreviewModeState> {
    let ((display, _), _) = combined_state;
    let display_guard = display.lock().await;
    let active = display_guard.is_in_preview_mode();
    Json(PreviewModeState { active })
}

// Updated handler for pinging preview mode
pub async fn ping_preview_mode(
    State(combined_state): State<CombinedState>,
    Json(ping_req): Json<PingPreviewRequest>,
) -> Result<StatusCode, StatusCode> {
    let ((display, _), _) = combined_state;
    let mut display_guard = display.lock().await;
    
    // Check if we're in preview mode
    if !display_guard.is_in_preview_mode() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Verify session ownership
    if !display_guard.is_preview_session_owner(&ping_req.session_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Update the ping time
    display_guard.update_preview_ping();
    Ok(StatusCode::OK)
}

// Handler for starting preview mode with a content item
pub async fn start_preview_mode(
    State(combined_state): State<CombinedState>,
    Json(start_req): Json<StartPreviewRequest>,
) -> Result<Json<PreviewModeResponse>, StatusCode> {
    let ((display, _), event_state) = combined_state;
    let mut display_guard = display.lock().await;
    
    // Check if a preview session is already active
    if display_guard.is_in_preview_mode() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Generate a session ID to identify this preview session
    let session_id = generate_uuid_string();
    
    // Broadcast that the editor is now locked
    let event_state_guard = event_state.lock().unwrap();
    event_state_guard.broadcast_editor_lock(true, Some(session_id.clone()));
    
    // Pass the session ID to the display manager
    display_guard.enter_preview_mode(start_req.item.clone(), session_id.clone());
    
    // Return the item that's being previewed along with the session ID
    Ok(Json(PreviewModeResponse {
        item: start_req.item,
        session_id,
    }))
}

// Handler to check if a session owns the lock
pub async fn check_session_owner(
    State(combined_state): State<CombinedState>,
    Json(request): Json<SessionCheckRequest>,
) -> Json<SessionCheckResponse> {
    let ((display, _), _) = combined_state;
    let display_guard = display.lock().await;
    
    // Check if this session ID matches the one in preview mode
    let is_owner = display_guard.is_preview_session_owner(&request.session_id);
    
    Json(SessionCheckResponse { is_owner })
}

// New handler for updating an existing preview
pub async fn update_preview(
    State(combined_state): State<CombinedState>,
    Json(update_req): Json<PreviewUpdateRequest>,
) -> Result<Json<PreviewModeResponse>, StatusCode> {
    let ((display, _), _) = combined_state;
    let mut display_guard = display.lock().await;
    
    // Check if this session owns the lock
    if !display_guard.is_in_preview_mode() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    if !display_guard.is_preview_session_owner(&update_req.session_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Update the preview content
    display_guard.update_preview_content(update_req.item.clone());
    
    // Return updated preview response
    Ok(Json(PreviewModeResponse {
        item: update_req.item,
        session_id: update_req.session_id,
    }))
}