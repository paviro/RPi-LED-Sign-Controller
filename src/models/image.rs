use serde::{Deserialize, Serialize};

fn default_scale() -> f32 {
    1.0
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageTransform {
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

impl Default for ImageTransform {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            scale: default_scale(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageKeyframe {
    pub timestamp_ms: u32,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageAnimation {
    pub keyframes: Vec<ImageKeyframe>,
    /// Number of times to loop the keyframe animation (None = infinite)
    pub iterations: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageContent {
    pub image_id: String,
    pub natural_width: u32,
    pub natural_height: u32,
    #[serde(default)]
    pub transform: ImageTransform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<ImageAnimation>,
}
