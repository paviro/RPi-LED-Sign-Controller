/// Provides shared configuration and helpers for all renderers
#[derive(Clone)]
pub struct RenderContext {
    /// Display width in pixels
    pub display_width: i32,
    
    /// Display height in pixels
    pub display_height: i32,
    
    /// User-defined brightness (0-100)
    pub brightness: u8,
}

impl RenderContext {
    /// Create a new render context
    pub fn new(display_width: i32, display_height: i32, brightness: u8) -> Self {
        Self {
            display_width,
            display_height,
            brightness,
        }
    }
    
    /// Apply brightness scaling to a color
    pub fn apply_brightness(&self, color: [u8; 3]) -> [u8; 3] {
        let brightness_scale = self.brightness as f32 / 100.0;
        [
            (color[0] as f32 * brightness_scale) as u8,
            (color[1] as f32 * brightness_scale) as u8,
            (color[2] as f32 * brightness_scale) as u8,
        ]
    }
    
    /// Calculate vertical position for centered text
    pub fn calculate_centered_text_position(&self, font_height: i32) -> i32 {
        let baseline_adjustment = 5;
        (self.display_height / 2) + (font_height / 2) - baseline_adjustment
    }
} 