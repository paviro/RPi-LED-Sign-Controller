use crate::models::border_effects::BorderEffect;
use crate::models::content::{ContentData, ContentDetails};
use crate::models::text::TextContent;
use crate::utils::uuid::generate_uuid_string;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub items: Vec<PlayListItem>,
    pub active_index: usize,
    pub repeat: bool,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            items: vec![], // Start with an empty playlist
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
    pub duration: Option<u64>, // Display duration in seconds (None = use repeat_count instead)
    pub repeat_count: Option<u32>, // Number of times to repeat (None = use duration instead)
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
                    "Both 'duration' and 'repeat_count' cannot be provided together",
                ));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "Either 'duration' or 'repeat_count' must be provided",
                ));
            }
            _ => {} // Exactly one is provided, which is valid
        }

        // Check for consistent configuration between content configuration and timing
        match &helper.content.data {
            ContentDetails::Text(text_content) => {
                if !text_content.scroll && helper.repeat_count.is_some() {
                    return Err(serde::de::Error::custom(
                        "When 'scroll' is false, 'duration' must be used instead of 'repeat_count'",
                    ));
                }
                if text_content.scroll && helper.duration.is_some() {
                    return Err(serde::de::Error::custom(
                        "When 'scroll' is true, 'repeat_count' must be used instead of 'duration'",
                    ));
                }
            }
            ContentDetails::Image(image_content) => {
                if image_content.image_id.trim().is_empty() {
                    return Err(serde::de::Error::custom(
                        "Image content requires a valid 'image_id'",
                    ));
                }
                if image_content.natural_width == 0 || image_content.natural_height == 0 {
                    return Err(serde::de::Error::custom(
                        "Image content requires non-zero natural dimensions",
                    ));
                }

                if let Some(animation) = &image_content.animation {
                    if animation.keyframes.len() < 2 {
                        return Err(serde::de::Error::custom(
                            "Animated images require at least two keyframes",
                        ));
                    }
                    if helper.duration.is_some() {
                        return Err(serde::de::Error::custom(
                            "Animated images must use 'repeat_count' instead of 'duration'",
                        ));
                    }
                } else if helper.duration.is_none() {
                    return Err(serde::de::Error::custom(
                        "Static images require 'duration' instead of 'repeat_count'",
                    ));
                }
            }
            ContentDetails::Clock(_) => {
                if helper.duration.is_none() {
                    return Err(serde::de::Error::custom(
                        "Clock content requires 'duration' instead of 'repeat_count'",
                    ));
                }
                if helper.repeat_count.is_some() {
                    return Err(serde::de::Error::custom(
                        "Clock content uses 'duration' instead of 'repeat_count'",
                    ));
                }
            }
            ContentDetails::Animation(animation_content) => {
                if helper.duration.is_none() {
                    return Err(serde::de::Error::custom(
                        "Animation content requires 'duration' instead of 'repeat_count'",
                    ));
                }
                if helper.repeat_count.is_some() {
                    return Err(serde::de::Error::custom(
                        "Animation content requires 'duration' and does not allow 'repeat_count'",
                    ));
                }
                if let Err(err) = animation_content.validate() {
                    return Err(serde::de::Error::custom(err));
                }
            }
        }

        // Determine whether repeat_count is required based on content
        let requires_repeat_count = match &helper.content.data {
            ContentDetails::Text(text_content) => text_content.scroll,
            ContentDetails::Image(image_content) => image_content.animation.is_some(),
            ContentDetails::Clock(_) => false,
            ContentDetails::Animation(_) => false,
        };

        // Check if repeat_count is required but missing
        if requires_repeat_count && helper.repeat_count.is_none() {
            let msg = match &helper.content.data {
                ContentDetails::Text(_) => {
                    "When 'scroll' is true, 'repeat_count' must be used instead of 'duration'"
                }
                ContentDetails::Image(_) => {
                    "Animated images require 'repeat_count' instead of 'duration'"
                }
                ContentDetails::Clock(_) => unreachable!(),
                ContentDetails::Animation(_) => {
                    "Animation content requires 'duration' instead of 'repeat_count'"
                }
            };
            return Err(serde::de::Error::custom(msg));
        }

        // Additional check: static content that shouldn't repeat_count
        if !requires_repeat_count && helper.repeat_count.is_some() {
            return Err(serde::de::Error::custom(
                "Repeat count can only be used with scrolling text or animated images",
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
            duration: Some(10), // Default to 10 seconds duration
            repeat_count: None, // No repeat count by default (exclusive with duration)
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
