use crate::models::{DisplayContent, Playlist, BorderEffect, ContentType};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    text::Text,
    pixelcolor::Rgb888,
    geometry::Point,
    Drawable,
};
use rpi_led_panel::{RGBMatrix, RGBMatrixConfig, HardwareMapping, Canvas};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use log::{info, error, debug};
use rand::Rng;

// Structure to manage our LED matrix state
pub struct DisplayManager {
    pub playlist: Playlist,
    matrix: RGBMatrix,
    pub canvas: Option<Box<Canvas>>,
    pub display_width: i32,
    pub text_width: i32,
    pub last_transition: Instant,
    pub current_repeat: u32,  // Track current repeat count
    pub scroll_position: i32, // Track scroll position
    pub completed_scrolls: u32, // Count completed scrolls
    pub border_animation_state: f32, // Animation state (0.0-1.0)
    pub last_animation_update: Instant,
}

impl DisplayManager {
    pub fn new() -> Self {
        Self::with_brightness(100)  // Start at full brightness
    }

    pub fn with_brightness(brightness: u8) -> Self {
        let brightness = brightness.clamp(0, 100);
        let config = RGBMatrixConfig {
            hardware_mapping: HardwareMapping::regular(),
            rows: 32,
            cols: 192,
            slowdown: Some(4),
            refresh_rate: 120,
            pwm_bits: 11,
            pwm_lsb_nanoseconds: 130,
            interlaced: false,
            dither_bits: 0,
            led_brightness: brightness,  // Set brightness in config
            ..RGBMatrixConfig::default()
        };

        let (matrix, canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
        
        let default_playlist = Playlist::default();
        
        Self {
            playlist: default_playlist,
            matrix,
            canvas: Some(canvas),
            display_width: 192,
            text_width: 0,
            last_transition: Instant::now(),
            current_repeat: 0,
            scroll_position: 192,
            completed_scrolls: 0,
            border_animation_state: 0.0,
            last_animation_update: Instant::now(),
        }
    }

    pub fn with_playlist(playlist: Playlist) -> Self {
        // Get brightness from the playlist instead of the active item
        let brightness = playlist.brightness;
        
        let config = RGBMatrixConfig {
            hardware_mapping: HardwareMapping::regular(),
            rows: 32,
            cols: 192,
            slowdown: Some(4),
            refresh_rate: 120,
            pwm_bits: 11,
            pwm_lsb_nanoseconds: 130,
            interlaced: false,
            dither_bits: 2,
            led_brightness: brightness,
            ..RGBMatrixConfig::default()
        };

        let (matrix, canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
        
        Self {
            playlist,
            matrix,
            canvas: Some(canvas),
            display_width: 192,
            text_width: 0,
            last_transition: Instant::now(),
            current_repeat: 0,
            scroll_position: 192,
            completed_scrolls: 0,
            border_animation_state: 0.0,
            last_animation_update: Instant::now(),
        }
    }

    pub fn calculate_text_width(&self, text: &str, _style: &MonoTextStyle<Rgb888>) -> i32 {
        // FONT_10X20 is 10 pixels wide per character, add a small buffer
        (text.len() as i32) * 10 + 2
    }

    pub fn get_current_content(&self) -> &DisplayContent {
        if self.playlist.items.is_empty() {
            // Return a default item with the help message
            // This is just a temporary reference - it's not stored in the playlist
            static DEFAULT_ITEM: Lazy<DisplayContent> = Lazy::new(|| DisplayContent {
                content_type: ContentType::Text,
                text: String::from("Adjust playlist on the web"),
                scroll: true,
                color: (0, 255, 0),  // Green color for visibility
                speed: 40.0,         // Slightly slower for readability
                duration: 0,
                repeat_count: 0,     // Infinite repeat
                border_effect: None, // No border effect for default item
                colored_segments: None, // No colored segments for default item
            });
            &DEFAULT_ITEM
        } else {
            &self.playlist.items[self.playlist.active_index]
        }
    }

    pub fn check_transition(&mut self) -> bool {
        // If playlist is empty, no transitions needed
        if self.playlist.items.is_empty() {
            return false;
        }

        // Clone necessary values to avoid borrowing issues
        let current_content = self.get_current_content().clone();
        let elapsed = self.last_transition.elapsed();
        
        // For duration-based items
        if current_content.duration > 0 && !current_content.scroll {
            let duration = Duration::from_secs(current_content.duration);
            if elapsed >= duration {
                self.current_repeat += 1;
                
                // Check if we've reached the repeat count
                if current_content.repeat_count == 0 || self.current_repeat < current_content.repeat_count {
                    // Reset the timer but stay on the same item
                    self.last_transition = Instant::now();
                    false
                } else {
                    // Move to the next item
                    self.advance_playlist();
                    true
                }
            } else {
                false
            }
        } else {
            // For scroll-based items, check if we've completed a scroll
            if self.completed_scrolls > 0 {
                self.current_repeat += 1;
                self.completed_scrolls = 0;
                
                // Check if we've reached the repeat count
                if current_content.repeat_count == 0 || self.current_repeat < current_content.repeat_count {
                    // Reset position but stay on the same item
                    self.scroll_position = self.display_width;
                    false
                } else {
                    // Move to the next item
                    self.advance_playlist();
                    true
                }
            } else {
                false
            }
        }
    }
    
    fn advance_playlist(&mut self) {
        // If playlist is empty, nothing to advance
        if self.playlist.items.is_empty() {
            return;
        }

        // Save current index
        let old_index = self.playlist.active_index;
        
        // Change to next item
        let length = self.playlist.items.len();
        if old_index + 1 < length {
            self.playlist.active_index = old_index + 1;
        } else if self.playlist.repeat {
            self.playlist.active_index = 0;
        }
        
        // Reset transition timestamp and counters
        self.last_transition = Instant::now();
        self.current_repeat = 0;
        self.completed_scrolls = 0;
        self.scroll_position = self.display_width;
    }

    pub fn update_animation_state(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_animation_update).as_secs_f32();
        self.last_animation_update = now;
        
        // Just accumulate time without the modulo operation
        // Allow it to grow continuously to handle multiple colors and long cycles
        self.border_animation_state += dt;
    }
    
    fn draw_border(&self, canvas: &mut Box<Canvas>, effect: &BorderEffect) {
        let height = 32; // Panel height
        let width = 192; // Panel width
        
        match effect {
            BorderEffect::None => {
                // No border effect
            },
            BorderEffect::Rainbow => {
                // Draw rainbow border
                for i in 0..width {
                    let hue = (i as f32 / width as f32 + self.border_animation_state) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    
                    // Top and bottom borders (2 pixels thick)
                    canvas.set_pixel(i as usize, 0, r, g, b);
                    canvas.set_pixel(i as usize, 1, r, g, b); // Second row for top
                    canvas.set_pixel(i as usize, (height - 1) as usize, r, g, b);
                    canvas.set_pixel(i as usize, (height - 2) as usize, r, g, b); // Second row for bottom
                }
                
                for i in 0..height {
                    let hue = (i as f32 / height as f32 + self.border_animation_state) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    
                    // Left and right borders (2 pixels thick)
                    canvas.set_pixel(0, i as usize, r, g, b);
                    canvas.set_pixel(1, i as usize, r, g, b); // Second column for left
                    canvas.set_pixel((width - 1) as usize, i as usize, r, g, b);
                    canvas.set_pixel((width - 2) as usize, i as usize, r, g, b); // Second column for right
                }
            },
            BorderEffect::Pulse { colors } => {
                // Get colors to use - either from the effect or default to text color
                let color_options = if colors.is_empty() {
                    let current = self.get_current_content();
                    vec![current.color]
                } else {
                    colors.clone()
                };
                
                // If we have no colors, don't render anything
                if color_options.is_empty() {
                    return;
                }
                
                // Speed up the animation by adjusting the time factor
                let adjusted_time = self.border_animation_state * 0.7;
                
                // Each color cycle: 2 seconds (1s fade in, 1s fade out)
                let seconds_per_color = 2.0;
                let total_cycle = seconds_per_color * color_options.len() as f32;
                
                // Figure out which color we're currently displaying
                let current_position = adjusted_time % total_cycle;
                let color_index = (current_position / seconds_per_color) as usize;
                
                // Safety check for array bounds
                if color_index >= color_options.len() {
                    return;
                }
                
                // Calculate brightness using a triangle wave
                let progress_in_color = (current_position % seconds_per_color) / seconds_per_color;
                
                let brightness = if progress_in_color < 0.5 {
                    progress_in_color * 2.0 // 0.0 -> 1.0
                } else {
                    (1.0 - progress_in_color) * 2.0 // 1.0 -> 0.0
                };
                
                // Get the color and apply brightness
                let (r, g, b) = color_options[color_index];
                let r = (r as f32 * brightness) as u8;
                let g = (g as f32 * brightness) as u8;
                let b = (b as f32 * brightness) as u8;
                
                // Draw the border (2 pixels thick)
                for i in 0..width {
                    // Top and bottom borders
                    canvas.set_pixel(i as usize, 0, r, g, b);
                    canvas.set_pixel(i as usize, 1, r, g, b); // Second row for top
                    canvas.set_pixel(i as usize, (height - 1) as usize, r, g, b);
                    canvas.set_pixel(i as usize, (height - 2) as usize, r, g, b); // Second row for bottom
                }
                
                for i in 0..height {
                    // Left and right borders
                    canvas.set_pixel(0, i as usize, r, g, b);
                    canvas.set_pixel(1, i as usize, r, g, b); // Second column for left
                    canvas.set_pixel((width - 1) as usize, i as usize, r, g, b);
                    canvas.set_pixel((width - 2) as usize, i as usize, r, g, b); // Second column for right
                }
            },
            BorderEffect::Sparkle { colors } => {
                // If no colors provided, use the text color as default
                let mut rng = rand::thread_rng();
                let color_options = if colors.is_empty() {
                    let current = self.get_current_content();
                    vec![current.color]
                } else {
                    colors.clone()
                };
                
                // Create sparkles based on animation state - increase count for thicker border
                for _ in 0..30 { // Increased from 20 to provide more density for 2-pixel border
                    // Randomly select one of the available colors
                    let color_index = rng.gen_range(0..color_options.len());
                    let (r, g, b) = color_options[color_index];
                    
                    // Random position along the border
                    let pos = rng.gen_range(0..2 * (width + height - 2));
                    let inner = rng.gen_bool(0.5); // 50% chance for inner or outer pixel
                    
                    if pos < width {
                        // Top border
                        let row = if inner { 1 } else { 0 };
                        canvas.set_pixel(pos as usize, row, r, g, b);
                    } else if pos < width * 2 {
                        // Bottom border
                        let row = if inner { height - 2 } else { height - 1 } as usize;
                        canvas.set_pixel((pos - width) as usize, row, r, g, b);
                    } else if pos < width * 2 + height - 2 {
                        // Left border (excluding corners)
                        let col = if inner { 1 } else { 0 };
                        canvas.set_pixel(col, (pos - width * 2 + 1) as usize, r, g, b);
                    } else {
                        // Right border (excluding corners)
                        let col = if inner { width - 2 } else { width - 1 } as usize;
                        canvas.set_pixel(col, (pos - (width * 2 + height - 2) + 1) as usize, r, g, b);
                    }
                }
            },
            BorderEffect::Gradient { colors } => {
                if colors.is_empty() {
                    return;
                }
                
                // Use at least 2 colors for gradient
                let colors = if colors.len() == 1 {
                    vec![colors[0], colors[0]]
                } else {
                    colors.clone()
                };
                
                let segments = colors.len();
                let perimeter = 2 * (width + height - 2);
                let segment_length = perimeter / segments;
                
                // Calculate offset for animation
                let offset = (self.border_animation_state * perimeter as f32) as usize;
                
                for pos in 0..perimeter {
                    // Apply offset and wrap around
                    let adjusted_pos = (pos + offset) % perimeter;
                    
                    // Determine which segment this position falls in
                    let segment_idx = adjusted_pos / segment_length;
                    let next_segment_idx = (segment_idx + 1) % segments;
                    
                    // Calculate interpolation factor within segment
                    let segment_progress = (adjusted_pos % segment_length) as f32 / segment_length as f32;
                    
                    // Get colors to interpolate between
                    let (r1, g1, b1) = colors[segment_idx];
                    let (r2, g2, b2) = colors[next_segment_idx];
                    
                    // Interpolate colors
                    let r = (r1 as f32 * (1.0 - segment_progress) + r2 as f32 * segment_progress) as u8;
                    let g = (g1 as f32 * (1.0 - segment_progress) + g2 as f32 * segment_progress) as u8;
                    let b = (b1 as f32 * (1.0 - segment_progress) + b2 as f32 * segment_progress) as u8;
                    
                    // Map position to actual pixel on display (2 pixels thick)
                    if pos < width {
                        // Top border
                        canvas.set_pixel(pos as usize, 0, r, g, b);
                        canvas.set_pixel(pos as usize, 1, r, g, b); // Second row
                    } else if pos < width * 2 {
                        // Bottom border
                        canvas.set_pixel((pos - width) as usize, (height - 1) as usize, r, g, b);
                        canvas.set_pixel((pos - width) as usize, (height - 2) as usize, r, g, b); // Second row
                    } else if pos < width * 2 + height - 2 {
                        // Left border (excluding corners)
                        canvas.set_pixel(0, (pos - width * 2 + 1) as usize, r, g, b);
                        canvas.set_pixel(1, (pos - width * 2 + 1) as usize, r, g, b); // Second column
                    } else {
                        // Right border (excluding corners)
                        canvas.set_pixel((width - 1) as usize, (pos - (width * 2 + height - 2) + 1) as usize, r, g, b);
                        canvas.set_pixel((width - 2) as usize, (pos - (width * 2 + height - 2) + 1) as usize, r, g, b); // Second column
                    }
                }
            }
        }
    }

    pub fn update_display(&mut self, position: i32) {
        let mut canvas = self.canvas.take().expect("Canvas missing");
        canvas.fill(0, 0, 0);  // Always clear the canvas
        
        let current = self.get_current_content().clone();  // Clone to avoid borrow issues
        let (r, g, b) = current.color;
        let default_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(r, g, b));
        
        self.text_width = self.calculate_text_width(&current.text, &default_text_style);
        
        // Adjust the vertical centering calculation
        let vertical_position = 22;  // This value centers most fonts better
        
        if current.colored_segments.is_some() && !current.colored_segments.as_ref().unwrap().is_empty() {
            // Render text with multiple colors
            let segments = current.colored_segments.as_ref().unwrap();
            let mut last_end = 0;
            
            for segment in segments {
                // Render the text segment if it's valid
                if segment.start < current.text.len() && segment.end <= current.text.len() && segment.start < segment.end {
                    // Render any text before this segment with default color if needed
                    if segment.start > last_end {
                        let default_segment = &current.text[last_end..segment.start];
                        let x_pos = if current.scroll {
                            position + (last_end as i32 * 10) // Approximate character width
                        } else {
                            (self.display_width - self.text_width) / 2 + (last_end as i32 * 10)
                        };
                        
                        Text::new(default_segment, Point::new(x_pos, vertical_position), default_text_style)
                            .draw(&mut *canvas)
                            .unwrap();
                    }
                    
                    // Render the colored segment
                    let segment_text = &current.text[segment.start..segment.end];
                    let (sr, sg, sb) = segment.color;
                    let segment_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(sr, sg, sb));
                    
                    let x_pos = if current.scroll {
                        position + (segment.start as i32 * 10) // Approximate character width
                    } else {
                        (self.display_width - self.text_width) / 2 + (segment.start as i32 * 10)
                    };
                    
                    Text::new(segment_text, Point::new(x_pos, vertical_position), segment_style)
                        .draw(&mut *canvas)
                        .unwrap();
                    
                    last_end = segment.end;
                }
            }
            
            // Render any remaining text with default color
            if last_end < current.text.len() {
                let remaining_text = &current.text[last_end..];
                let x_pos = if current.scroll {
                    position + (last_end as i32 * 10) // Approximate character width
                } else {
                    (self.display_width - self.text_width) / 2 + (last_end as i32 * 10)
                };
                
                Text::new(remaining_text, Point::new(x_pos, vertical_position), default_text_style)
                    .draw(&mut *canvas)
                    .unwrap();
            }
        } else {
            // Render text with a single color (existing code)
            if current.scroll {
                Text::new(&current.text, Point::new(position, vertical_position), default_text_style)
                    .draw(&mut *canvas)
                    .unwrap();
            } else {
                let x = (self.display_width - self.text_width) / 2;
                Text::new(&current.text, Point::new(x, vertical_position), default_text_style)
                    .draw(&mut *canvas)
                    .unwrap();
            }
        }
        
        // Draw border effect if one is specified
        if let Some(effect) = &current.border_effect {
            if effect != &BorderEffect::None {
                self.draw_border(&mut canvas, effect);
            }
        }
        
        let updated_canvas = self.matrix.update_on_vsync(canvas);
        self.canvas = Some(updated_canvas);
    }

    pub fn reinitialize_with_brightness(&mut self, brightness: u8) {
        let brightness = brightness.clamp(0, 100);
        info!("Reinitializing display with brightness: {}", brightness);
        
        let config = RGBMatrixConfig {
            hardware_mapping: HardwareMapping::regular(),
            rows: 32,
            cols: 192,
            slowdown: Some(4),
            refresh_rate: 120,
            pwm_bits: 11,
            pwm_lsb_nanoseconds: 130,
            interlaced: false,
            dither_bits: 2,
            led_brightness: brightness,
            ..RGBMatrixConfig::default()
        };

        match RGBMatrix::new(config, 0) {
            Ok((matrix, canvas)) => {
                debug!("Matrix reinitialized successfully");
                self.matrix = matrix;
                self.canvas = Some(canvas);
                
                // Update global brightness in the playlist
                self.playlist.brightness = brightness;
            },
            Err(e) => {
                error!("Failed to reinitialize matrix: {}", e);
                // Continue using the existing matrix rather than crashing
            }
        }
    }

    // Add a method to check if the playlist is empty
    #[allow(dead_code)]
    pub fn is_playlist_empty(&self) -> bool {
        self.playlist.items.is_empty()
    }

    // Add a method to get the current brightness
    pub fn get_brightness(&self) -> u8 {
        self.playlist.brightness
    }
}

// Add this helper function for HSV to RGB conversion (outside the impl block)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    
    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;
    
    (r, g, b)
} 