use crate::display::manager::DisplayManager;
use crate::models::content::ContentDetails;
use crate::web::api::events::EventState;
use log::info;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

// Display loop function that manages the update cycle
pub async fn display_loop(
    display: Arc<tokio::sync::Mutex<DisplayManager>>,
    event_state: Arc<Mutex<EventState>>,
) {
    info!("Starting display update loop");
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
        if let Some(_session_id) = display_guard.check_preview_timeout(PREVIEW_TIMEOUT) {
            // If preview timed out, broadcast the editor unlock event
            if let Ok(event_state_guard) = event_state.lock() {
                event_state_guard.broadcast_editor_lock(false, None);
            }
        }

        // Check if transition to next item is needed
        let transition_occurred = display_guard.check_transition();
        if transition_occurred {
            let current = display_guard.get_current_content();
            let index = display_guard.playlist.active_index;
            let total = display_guard.playlist.items.len();

            // Get content description
            let content_desc = match &current.content.data {
                ContentDetails::Text(text_content) => {
                    let preview = if text_content.text.len() > 30 {
                        format!("{}...", &text_content.text[..27])
                    } else {
                        text_content.text.clone()
                    };
                    format!("Text: \"{}\"", preview)
                }
                ContentDetails::Image(image_content) => format!(
                    "Image: {} ({}x{})",
                    image_content.image_id,
                    image_content.natural_width,
                    image_content.natural_height
                ),
            };

            info!(
                "Transitioned to playlist item {} of {}: {}",
                index + 1,
                total,
                content_desc
            );
        }

        // Update the renderers with the elapsed time
        display_guard.update_renderer(dt);

        // Update the display
        display_guard.update_display();

        drop(display_guard);

        // Log performance stats periodically
        frame_count += 1;
        if now.duration_since(last_stats_time).as_secs() >= 60 {
            // Log every minute
            let fps = frame_count as f32 / now.duration_since(last_stats_time).as_secs_f32();
            info!("Display performance: {:.1} FPS", fps);
            frame_count = 0;
            last_stats_time = now;
        }

        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}
