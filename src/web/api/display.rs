use axum::{extract::State, Json};
use serde::Serialize;

use crate::web::api::CombinedState;

#[derive(Serialize)]
pub struct DisplayInfoResponse {
    pub width: i32,
    pub height: i32,
}

pub async fn get_display_info(
    State(combined_state): State<CombinedState>,
) -> Json<DisplayInfoResponse> {
    let ((display, _storage), _events) = combined_state;
    let display_guard = display.lock().await;
    Json(DisplayInfoResponse {
        width: display_guard.display_width,
        height: display_guard.display_height,
    })
}
