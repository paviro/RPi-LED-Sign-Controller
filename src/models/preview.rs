use serde::{Deserialize, Serialize};

// New structure for preview mode state
#[derive(Serialize, Deserialize)]
pub struct PreviewModeState {
    pub active: bool,
}
