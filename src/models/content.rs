use serde::{Deserialize, Serialize};
use crate::models::text::TextContent;

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