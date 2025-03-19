use serde::{Deserialize, Serialize};
use rpi_led_panel::RGBMatrixConfig;

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
    pub content_type: ContentType,  // New field for content type
    pub text: String,
    pub scroll: bool,
    pub color: (u8, u8, u8),       // Default text color
    pub speed: f32,                // Pixels per second
    pub duration: u64,             // Display duration in seconds (0 = indefinite)
    pub repeat_count: u32,         // Number of times to repeat (0 = indefinite)
    pub border_effect: Option<BorderEffect>, // Optional border effect
    pub colored_segments: Option<Vec<ColoredSegment>>, // New field for multi-colored text
}

// Optionally update the default implementation if needed
impl Default for DisplayContent {
    fn default() -> Self {
        Self {
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
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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

// New struct to represent a colored segment within the text
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColoredSegment {
    pub start: usize,  // Start index in the text
    pub end: usize,    // End index in the text (exclusive)
    pub color: (u8, u8, u8), // RGB color for this segment
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub items: Vec<DisplayContent>,
    pub active_index: usize,
    pub repeat: bool,
    pub brightness: u8,  // Global brightness setting
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            items: vec![],  // Start with an empty playlist
            active_index: 0,
            repeat: true,
            brightness: 100,  // Default brightness
        }
    }
}

/// Our own configuration structure that stores just what we need
#[derive(Clone, Debug)]
pub struct DisplayConfig {
    pub rows: usize,           // Changed from u32 to usize
    pub cols: usize,           // Changed from u32 to usize
    pub chain_length: usize,   // Changed from u32 to usize
    pub parallel: usize,       // Changed from u32 to usize
    pub brightness: u8,
}

impl DisplayConfig {
    /// Create from command-line args
    pub fn from_args() -> Self {
        let matrix_config: RGBMatrixConfig = argh::from_env();
        
        Self {
            rows: matrix_config.rows,
            cols: matrix_config.cols,
            chain_length: matrix_config.chain_length,
            parallel: matrix_config.parallel,
            brightness: matrix_config.led_brightness,
        }
    }
    
    /// Create a matrix config from our display config
    pub fn to_matrix_config(&self) -> RGBMatrixConfig {
        let mut config: RGBMatrixConfig = argh::from_env();
        config.led_brightness = self.brightness;
        config
    }
    
    /// Calculate the total display width in pixels
    pub fn display_width(&self) -> i32 {
        (self.cols * self.chain_length) as i32
    }
    
    /// Calculate the total display height in pixels
    pub fn display_height(&self) -> i32 {
        (self.rows * self.parallel) as i32
    }
} 