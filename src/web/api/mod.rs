use std::sync::Arc;
use crate::display::manager::DisplayManager;
use crate::storage::app_storage::SharedStorage;

pub mod playlist;
pub mod settings;
pub mod preview;

// Type alias for our application state
type AppState = (Arc<tokio::sync::Mutex<DisplayManager>>, SharedStorage);