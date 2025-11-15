use crate::models::animation::AnimationContent;
use crate::models::clock::ClockContent;
use crate::models::image::ImageContent;
use crate::models::text::TextContent;
use serde::{Deserialize, Serialize};

// Add a ContentType enum to models.rs
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ContentType {
    Text,
    Image,
    Animation,
    Clock,
}

// Provide default implementation
impl Default for ContentType {
    fn default() -> Self {
        ContentType::Text
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
    Image(ImageContent),
    Animation(AnimationContent),
    Clock(ClockContent),
}
