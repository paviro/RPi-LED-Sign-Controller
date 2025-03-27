use std::sync::Arc;
use crate::display::manager::DisplayManager;
use crate::storage::app_storage::SharedStorage;
use crate::web::api::events::SharedEventState;

pub mod playlist;
pub mod settings;
pub mod preview;
pub mod events;

// Type alias for our application state
pub type AppState = (Arc<tokio::sync::Mutex<DisplayManager>>, SharedStorage);
// Combined state type including SSE state
pub type CombinedState = (AppState, SharedEventState);