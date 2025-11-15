use serde::{Deserialize, Serialize};

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
    pub start: usize,           // Start index in the text (character position)
    pub end: usize,             // End index in the text (exclusive, character position)
    pub color: Option<[u8; 3]>, // Changed from tuple to array
    pub formatting: Option<TextFormatting>, // Optional formatting
}

// Text-specific content structure
#[derive(Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub scroll: bool,
    pub color: [u8; 3], // Changed from tuple to array
    pub speed: f32,
    pub text_segments: Option<Vec<TextSegment>>,
}
