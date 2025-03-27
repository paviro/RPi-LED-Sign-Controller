use serde::{Deserialize, Serialize};
// New structure for brightness settings
#[derive(Serialize, Deserialize, Clone)]
pub struct BrightnessSettings {
    pub brightness: u8,
}

// New structure for reordering request
#[derive(Deserialize)]
pub struct ReorderRequest {
    pub item_ids: Vec<String>,
}

