use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde::ser::{Serializer, SerializeMap};

// Add a ContentType enum to models.rs
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ContentType {
    Text,
    // Future types will be added here (Image, Clock, Animation, etc.)
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
    pub color: Option<[u8; 3]>,   // Changed from tuple to array
    pub formatting: Option<TextFormatting>,  // Optional formatting
}

// Base structure for all display content items
#[derive(Clone, Serialize)]
pub struct DisplayContent {
    #[serde(default = "generate_uuid_string")]
    pub id: String,
    pub duration: Option<u64>,      // Display duration in seconds (None = use repeat_count instead)
    pub repeat_count: Option<u32>,  // Number of times to repeat (None = use duration instead)
    pub border_effect: Option<BorderEffect>, // Optional border effect
    pub content: ContentData,
}

// Custom deserialization to enforce mutual exclusivity and scroll validation
impl<'de> Deserialize<'de> for DisplayContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default = "generate_uuid_string")]
            id: String,
            duration: Option<u64>,
            repeat_count: Option<u32>,
            border_effect: Option<BorderEffect>,
            content: ContentData,
        }

        let helper = Helper::deserialize(deserializer)?;

        // Check that exactly one of duration or repeat_count is provided
        match (helper.duration, helper.repeat_count) {
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(
                    "Both 'duration' and 'repeat_count' cannot be provided together"
                ));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "Either 'duration' or 'repeat_count' must be provided"
                ));
            }
            _ => {} // Exactly one is provided, which is valid
        }

        // Extract the text content - using match instead of if let to prepare for future types
        let requires_repeat_count = match &helper.content.data {
            ContentDetails::Text(text_content) => text_content.scroll,
            // Add future content types here
        };

        // Check if repeat_count is required but missing
        if requires_repeat_count && helper.repeat_count.is_none() {
            return Err(serde::de::Error::custom(
                "When 'scroll' is true, 'repeat_count' must be used instead of 'duration'"
            ));
        }

        Ok(DisplayContent {
            id: helper.id,
            duration: helper.duration,
            repeat_count: helper.repeat_count,
            border_effect: helper.border_effect,
            content: helper.content,
        })
    }
}

// Tagged union approach for different content types
#[derive(Clone, Serialize, Deserialize)]
pub struct ContentData {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub data: ContentDetails,
}

// Content details as an enum
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentDetails {
    Text(TextContent),
    // Future types: Image, Clock, etc.
}

// Text-specific content structure
#[derive(Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub scroll: bool,
    pub color: [u8; 3],  // Changed from tuple to array
    pub speed: f32,
    pub text_segments: Option<Vec<TextSegment>>,
}

// Helper function to generate UUID strings for default values
fn generate_uuid_string() -> String {
    Uuid::new_v4().to_string()
}

// Default implementation for DisplayContent
impl Default for DisplayContent {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            duration: Some(10),    // Default to 10 seconds duration
            repeat_count: None,    // No repeat count by default (exclusive with duration)
            border_effect: None,
            content: ContentData {
                content_type: ContentType::Text,
                data: ContentDetails::Text(TextContent {
                    text: String::new(),
                    scroll: true,
                    color: [255, 255, 255],
                    speed: 50.0,
                    text_segments: None,
                }),
            },
        }
    }
}

// New enum for available border effects
#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum BorderEffect {
    None,
    Rainbow,
    Pulse { colors: Vec<[u8; 3]> }, // Changed from tuple to array
    Sparkle { colors: Vec<[u8; 3]> }, // Changed from tuple to array
    Gradient { colors: Vec<[u8; 3]> }, // Changed from tuple to array
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