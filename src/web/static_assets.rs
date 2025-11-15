use crate::utils::static_assets::StaticAssets;
use axum::{
    extract::{Path, Request},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use log::{debug, warn};

// Update index handler to use embedded assets and properly handle ownership
pub async fn index_handler() -> Html<String> {
    let index_html =
        StaticAssets::get("index.html").expect("index.html not found in embedded assets");

    // Convert the content to string
    let content = std::str::from_utf8(index_html.data.as_ref())
        .expect("Failed to convert index.html to UTF-8")
        .to_string();

    // Process the content to fix paths if needed
    // Note: This is a basic approach - you might need more sophisticated HTML parsing
    let processed_content = content;

    Html(processed_content)
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
                content.data,
            )
                .into_response()
        }
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
                content.data,
            )
                .into_response()
        }
        None => {
            warn!("Static asset not found: {}", path);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
