use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde::ser::{Serializer, SerializeMap};

// Add a ContentType enum to models.rs
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ContentType {
    Text,
    // Future types will be added here (Image, Video, Animation, etc.)
}

// Provide default implementation
impl Default for ContentType {
    fn default() -> Self {
        ContentType::Text
    }
}

// Structure to hold display content configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayContent {
    pub id: String,             // New unique ID field
    pub content_type: ContentType,
    pub text: String,
    pub scroll: bool,
    pub color: (u8, u8, u8),    // Default text color
    pub speed: f32,             // Pixels per second
    pub duration: u64,          // Display duration in seconds (0 = indefinite)
    pub repeat_count: u32,      // Number of times to repeat (0 = indefinite)
    pub border_effect: Option<BorderEffect>, // Optional border effect
    pub colored_segments: Option<Vec<ColoredSegment>>, // New field for multi-colored text
}

// Optionally update the default implementation if needed
impl Default for DisplayContent {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content_type: ContentType::Text,
            text: String::new(),
            scroll: true,
            color: (255, 255, 255),
            speed: 50.0,
            duration: 10,
            repeat_count: 1,
            border_effect: None,
            colored_segments: None,
        }
    }
}

// New enum for available border effects
#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum BorderEffect {
    None,
    Rainbow,
    Pulse { colors: Vec<(u8, u8, u8)> },
    Sparkle { colors: Vec<(u8, u8, u8)> },
    Gradient { colors: Vec<(u8, u8, u8)> },
}

// Provide defaults
impl Default for BorderEffect {
    fn default() -> Self {
        BorderEffect::None
    }
}

// Updated structure to represent a colored segment within the text
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColoredSegment {
    // Original index-based fields kept for compatibility
    pub start: usize,  // Start index in the text
    pub end: usize,    // End index in the text (exclusive)
    
    // New fields for the API
    pub text: Option<String>,  // The text segment content
    pub color: (u8, u8, u8),   // RGB color for this segment
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub items: Vec<DisplayContent>,
    pub active_index: usize,
    pub repeat: bool,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            items: vec![],  // Start with an empty playlist
            active_index: 0,
            repeat: true,
        }
    }
}

// New structure for reordering request
#[derive(Deserialize)]
pub struct ReorderRequest {
    pub item_ids: Vec<String>,
}

// New structure for brightness settings
#[derive(Serialize, Deserialize)]
pub struct BrightnessSettings {
    pub brightness: u8,
}

impl Serialize for BorderEffect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Simple variants get serialized as {"Variant": null}
            BorderEffect::None => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("None", &Option::<()>::None)?;
                map.end()
            },
            BorderEffect::Rainbow => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Rainbow", &Option::<()>::None)?;
                map.end()
            },
            // Complex variants continue using the default serialization
            BorderEffect::Pulse { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Pulse", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
            BorderEffect::Sparkle { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Sparkle", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
            BorderEffect::Gradient { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Gradient", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
        }
    }
}