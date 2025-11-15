use crate::models::settings::BrightnessSettings;
use crate::web::api::CombinedState;
use axum::extract::State;
use axum::Json;
use log::info;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// New handler to get the current brightness
pub async fn get_brightness(
    State(combined_state): State<CombinedState>,
) -> Json<BrightnessSettings> {
    let ((display, _), _) = combined_state;
    let display = display.lock().await;

    let brightness = display.get_brightness();

    Json(BrightnessSettings { brightness })
}

// Handler for updating brightness - applies brightness through color scaling
pub async fn update_brightness(
    State(combined_state): State<CombinedState>,
    Json(settings): Json<BrightnessSettings>,
) -> Json<BrightnessSettings> {
    // Initialize static variables on first call
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    static LAST_BRIGHTNESS: AtomicU8 = AtomicU8::new(0);
    static LAST_UPDATE_TIME: AtomicI64 = AtomicI64::new(0);
    static SAVE_PENDING: AtomicBool = AtomicBool::new(false);
    static LATEST_TASK_ID: AtomicI64 = AtomicI64::new(0);

    // Destructure the state
    let ((display, storage), sse_state) = combined_state;

    // Get current timestamp in milliseconds
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    // Always update the display immediately
    let mut display = display.lock().await;

    // Initialize the static variable on first call
    if !INITIALIZED.load(Ordering::SeqCst) {
        LAST_BRIGHTNESS.store(display.get_brightness(), Ordering::SeqCst);
        INITIALIZED.store(true, Ordering::SeqCst);
    }

    display.set_brightness(settings.brightness);

    // Update tracking for brightness
    let prev_brightness = LAST_BRIGHTNESS.swap(settings.brightness, Ordering::SeqCst);
    LAST_UPDATE_TIME.store(now, Ordering::SeqCst);

    // If brightness changed, mark save as pending
    if prev_brightness != settings.brightness {
        SAVE_PENDING.store(true, Ordering::SeqCst);

        // Only log if brightness changed by more than 10 or is at min/max
        if (settings.brightness as i16 - prev_brightness as i16).abs() >= 10
            || settings.brightness == 0
            || settings.brightness == 100
        {
            // Add more descriptive logging with percentages
            info!(
                "Display brightness: {}% -> {}%",
                prev_brightness, settings.brightness
            );
        }

        // Broadcast the brightness change via SSE
        let sse_state_guard = sse_state.lock().unwrap();
        sse_state_guard.broadcast_brightness(BrightnessSettings {
            brightness: settings.brightness,
        });

        // Get current brightness for the task
        let brightness = settings.brightness;

        // Clone storage for use in the task
        let storage_clone = storage.clone();

        // Increment the task ID and get its value
        let task_id = LATEST_TASK_ID.fetch_add(1, Ordering::SeqCst) + 1;

        tokio::spawn(async move {
            // Wait 1 second
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Check if there have been no updates during our waiting period
            let last_update = LAST_UPDATE_TIME.load(Ordering::SeqCst);
            let time_passed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
                - last_update;

            // Only save if this is still the latest task and save is pending
            let is_latest = LATEST_TASK_ID.load(Ordering::SeqCst) == task_id;

            // If no updates for ~1 second, save is still pending, and this is the latest task
            if time_passed >= 950 && SAVE_PENDING.load(Ordering::SeqCst) && is_latest {
                // Reset pending flag
                SAVE_PENDING.store(false, Ordering::SeqCst);

                if let Ok(storage_guard) = storage_clone.lock() {
                    storage_guard.save_brightness(brightness);
                }
            }
        });
    }

    // Return the updated settings
    Json(BrightnessSettings {
        brightness: display.get_brightness(),
    })
}
