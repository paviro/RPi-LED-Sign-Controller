use serde::{Deserialize, Serialize};
use crate::models::border_effects::BorderEffect;
use crate::models::content::{ContentData, ContentDetails};
use crate::models::text::TextContent;
use crate::utils::uuid::generate_uuid_string;

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub items: Vec<PlayListItem>,
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

// Base structure for all display content items
#[derive(Clone, Serialize)]
pub struct PlayListItem {
    #[serde(default = "generate_uuid_string")]
    pub id: String,
    pub duration: Option<u64>,      // Display duration in seconds (None = use repeat_count instead)
    pub repeat_count: Option<u32>,  // Number of times to repeat (None = use duration instead)
    pub border_effect: Option<BorderEffect>, // Optional border effect
    pub content: ContentData,
}

// Custom deserialization to enforce mutual exclusivity and scroll validation
impl<'de> Deserialize<'de> for PlayListItem {
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

        // Check for consistent configuration between scroll and duration/repeat_count
        match &helper.content.data {
            ContentDetails::Text(text_content) => {
                if !text_content.scroll && helper.repeat_count.is_some() {
                    return Err(serde::de::Error::custom(
                        "When 'scroll' is false, 'duration' must be used instead of 'repeat_count'"
                    ));
                }
                if text_content.scroll && helper.duration.is_some() {
                    return Err(serde::de::Error::custom(
                        "When 'scroll' is true, 'repeat_count' must be used instead of 'duration'"
                    ));
                }
            },
            // Handle other content types as needed
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

        Ok(PlayListItem {
            id: helper.id,
            duration: helper.duration,
            repeat_count: helper.repeat_count,
            border_effect: helper.border_effect,
            content: helper.content,
        })
    }
}

// Default implementation for PlayListItem
impl Default for PlayListItem {
    fn default() -> Self {
        Self {
            id: generate_uuid_string(),
            duration: Some(10),    // Default to 10 seconds duration
            repeat_count: None,    // No repeat count by default (exclusive with duration)
            border_effect: None,
            content: ContentData {
                content_type: crate::models::content::ContentType::Text,
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