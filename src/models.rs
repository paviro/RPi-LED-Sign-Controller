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

// Text formatting flags structure with explicit defaults
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextFormatting {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
}

// Implement default manually to be explicit
impl Default for TextFormatting {
    fn default() -> Self {
        Self {
            bold: false,
            underline: false,
            strikethrough: false,
        }
    }
}

// New structure to represent a text segment with optional formatting and color
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextSegment {
    pub start: usize,  // Start index in the text (character position)
    pub end: usize,    // End index in the text (exclusive, character position)
    pub color: Option<(u8, u8, u8)>,   // Optional color (use parent text color if None)
    pub formatting: Option<TextFormatting>,  // Optional formatting
}

// Structure to hold display content configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayContent {
    #[serde(default = "generate_uuid_string")]
    pub id: String,             // ID field with default function
    pub content_type: ContentType,
    pub text: String,          // Full text (for backwards compatibility)
    pub scroll: bool,
    pub color: (u8, u8, u8),    // Default text color
    pub speed: f32,             // Pixels per second
    pub duration: u64,          // Display duration in seconds (0 = indefinite)
    pub repeat_count: u32,      // Number of times to repeat (0 = indefinite)
    pub border_effect: Option<BorderEffect>, // Optional border effect
    pub text_segments: Option<Vec<TextSegment>>, // New field replacing colored_segments
}

// Helper function to generate UUID strings for default values
fn generate_uuid_string() -> String {
    Uuid::new_v4().to_string()
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
            text_segments: None,
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

// New structure for preview mode state
#[derive(Serialize, Deserialize)]
pub struct PreviewModeState {
    pub active: bool,
}