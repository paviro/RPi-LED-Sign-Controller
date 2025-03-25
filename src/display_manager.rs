use crate::models::{DisplayContent, Playlist, BorderEffect, ContentType, ContentData, TextContent, ContentDetails};
use embedded_graphics::mono_font::iso_8859_1::FONT_10X20 as FONT_10X20_LATIN1;
use embedded_graphics::{
    mono_font::{MonoTextStyle},
    text::{Text},
    pixelcolor::Rgb888,
    geometry::Point,
    Drawable,
};
use std::time::Instant;
use once_cell::sync::Lazy;
use log::{info, debug};
use rand::Rng;
use crate::led_driver::{LedDriver, LedCanvas};
use crate::embedded_graphics_support::EmbeddedGraphicsCanvas;
use crate::config::DisplayConfig;
use uuid::Uuid;

// Structure to manage our LED matrix state
pub struct DisplayManager {
    pub playlist: Playlist,
    driver: Box<dyn LedDriver>,
    pub canvas: Option<Box<dyn LedCanvas>>,
    pub display_width: i32,
    pub display_height: i32,
    pub text_width: i32,
    pub last_transition: Instant,
    pub current_repeat: u32,  // Track current repeat count
    pub scroll_position: i32, // Track scroll position
    pub completed_scrolls: u32, // Count completed scrolls
    pub border_animation_state: f32, // Animation state (0.0-1.0)
    pub last_animation_update: Instant,
    config: DisplayConfig, // Our clearer config object
    // Add preview mode fields
    preview_mode: bool,
    preview_content: Option<DisplayContent>,
    last_preview_ping: Instant,
}

impl DisplayManager {
    pub fn with_config_and_driver(config: &DisplayConfig, driver: Box<dyn LedDriver>) -> Self {
        // Get display dimensions
        let display_width = config.display_width();
        let display_height = config.display_height();
        
        info!("Initializing display: {}x{} (rows={}, cols={}, chain={}, parallel={})",
              display_width, display_height, config.rows, config.cols, 
              config.chain_length, config.parallel);
        
        // Get the canvas from the driver
        let mut driver_box = driver;
        let canvas = driver_box.take_canvas();
        
        // Get default playlist
        let default_playlist = Playlist::default();
        
        Self {
            playlist: default_playlist,
            driver: driver_box,
            canvas,
            display_width,
            display_height,
            text_width: 0,
            last_transition: Instant::now(),
            current_repeat: 0,
            scroll_position: display_width,
            completed_scrolls: 0,
            border_animation_state: 0.0,
            last_animation_update: Instant::now(),
            config: config.clone(),
            // Initialize preview mode fields
            preview_mode: false,
            preview_content: None,
            last_preview_ping: Instant::now(),
        }
    }

    pub fn with_playlist_config_and_driver(playlist: Playlist, config: &DisplayConfig, driver: Box<dyn LedDriver>) -> Self {
        // Get dimensions
        let display_width = config.display_width();
        let display_height = config.display_height();
        
        info!("Initializing display with playlist: {}x{} (rows={}, cols={}, chain={}, parallel={})",
              display_width, display_height, config.rows, config.cols, 
              config.chain_length, config.parallel);
        
        // Get the canvas from the driver
        let mut driver_box = driver;
        let canvas = driver_box.take_canvas();
        
        Self {
            playlist,
            driver: driver_box,
            canvas,
            display_width,
            display_height,
            text_width: 0,
            last_transition: Instant::now(),
            current_repeat: 0,
            scroll_position: display_width,
            completed_scrolls: 0,
            border_animation_state: 0.0,
            last_animation_update: Instant::now(),
            config: config.clone(),
            // Initialize preview mode fields
            preview_mode: false,
            preview_content: None,
            last_preview_ping: Instant::now(),
        }
    }

    pub fn calculate_text_width(&self, text: &str, _style: &MonoTextStyle<Rgb888>) -> i32 {
        // Count actual characters, not bytes
        (text.chars().count() as i32) * 10 + 2
    }

    pub fn get_current_content(&self) -> &DisplayContent {
        // If we're in preview mode, show the preview content
        if self.preview_mode && self.preview_content.is_some() {
            return self.preview_content.as_ref().unwrap();
        }
        
        if self.playlist.items.is_empty() {
            // Store the default message item
            static DEFAULT_ITEM: Lazy<DisplayContent> = Lazy::new(|| {
                // Get the local IP for a more helpful message
                let ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());
                
                DisplayContent {
                    id: Uuid::new_v4().to_string(),
                    duration: None,                   // Updated to use None
                    repeat_count: Some(0),            // Infinite repeat with Some(0)
                    border_effect: Some(BorderEffect::Pulse { 
                        colors: vec![[0, 255, 0], [0, 200, 0]] 
                    }),
                    content: ContentData {
                        content_type: ContentType::Text,
                        data: ContentDetails::Text(TextContent {
                            text: format!("LED Matrix Controller | Web interface: http://{}:3000 | Use web UI to configure display", ip),
                            scroll: true,
                            color: [0, 255, 0],  // Green color for visibility
                            speed: 30.0,         // Slower for better readability
                            text_segments: None,
                        }),
                    },
                }
            });
            &DEFAULT_ITEM
        } else {
            &self.playlist.items[self.playlist.active_index]
        }
    }

    pub fn check_transition(&mut self) -> bool {
        // Skip transitions when in preview mode
        if self.preview_mode {
            return false;
        }
        
        // If playlist is empty, no transitions needed
        if self.playlist.items.is_empty() {
            return false;
        }

        let now = Instant::now();
        let current = self.get_current_content();
        
        // Handle transition logic based on which field is set
        let should_transition = match (current.duration, current.repeat_count) {
            (Some(duration), None) => {
                // Duration-based transition (in seconds)
                if duration > 0 && now.duration_since(self.last_transition).as_secs() >= duration {
                    true
                } else {
                    false
                }
            },
            (None, Some(repeat_count)) => {
                // Repeat-count based transition
                if repeat_count > 0 && self.completed_scrolls >= repeat_count {
                    true
                } else {
                    false
                }
            },
            _ => false, // This should never happen due to our validation
        };
        
        if should_transition {
            self.advance_playlist();
            return true;
        }
        
        false
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
    
    fn draw_border(&self, canvas: &mut Box<dyn LedCanvas>, effect: &BorderEffect) {
        let height = self.display_height; // Use calculated display height
        let width = self.display_width; // Use calculated display width
        
        match effect {
            BorderEffect::None => {
                // No border effect
            },
            BorderEffect::Rainbow => {
                // Draw rainbow border
                for i in 0..width {
                    let hue = (i as f32 / width as f32 + self.border_animation_state) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    // Apply brightness scaling
                    let [r, g, b] = self.apply_brightness([r, g, b]);
                    
                    // Top and bottom borders (2 pixels thick)
                    canvas.set_pixel(i as usize, 0, r, g, b);
                    canvas.set_pixel(i as usize, 1, r, g, b); // Second row for top
                    canvas.set_pixel(i as usize, (height - 1) as usize, r, g, b);
                    canvas.set_pixel(i as usize, (height - 2) as usize, r, g, b); // Second row for bottom
                }
                
                for i in 0..height {
                    let hue = (i as f32 / height as f32 + self.border_animation_state) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    // Apply brightness scaling
                    let [r, g, b] = self.apply_brightness([r, g, b]);
                    
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
                    // Extract color from current content using pattern matching
                    let current = self.get_current_content();
                    match &current.content.data {
                        ContentDetails::Text(text_content) => vec![text_content.color],
                        #[allow(unreachable_patterns)]
                        _ => vec![] // Empty vec for future content types
                    }
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
                
                let effect_brightness = if progress_in_color < 0.5 {
                    progress_in_color * 2.0 // 0.0 -> 1.0
                } else {
                    (1.0 - progress_in_color) * 2.0 // 1.0 -> 0.0
                };
                
                // Get the color and pre-scale it for the pulse effect
                let [r, g, b] = color_options[color_index];
                let pre_scaled = [
                    (r as f32 * effect_brightness) as u8,
                    (g as f32 * effect_brightness) as u8,
                    (b as f32 * effect_brightness) as u8
                ];
                
                // Then apply user brightness scaling using our consistent method
                let [r, g, b] = self.apply_brightness(pre_scaled);
                
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
                    // Extract color from current content using pattern matching
                    let current = self.get_current_content();
                    match &current.content.data {
                        ContentDetails::Text(text_content) => vec![text_content.color],
                        #[allow(unreachable_patterns)]
                        _ => vec![] // Empty vec for future content types
                    }
                } else {
                    colors.clone()
                };
                
                // Create sparkles based on animation state - increase count for thicker border
                for _ in 0..30 { // Increased from 20 to provide more density for 2-pixel border
                    // Randomly select one of the available colors and apply brightness
                    let color_index = rng.gen_range(0..color_options.len());
                    let [r, g, b] = self.apply_brightness(color_options[color_index]);
                    
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
                let perimeter = 2 * ((width as usize) + (height as usize) - 2);
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
                    let [r1, g1, b1] = colors[segment_idx];
                    let [r2, g2, b2] = colors[next_segment_idx];
                    
                    // Interpolate colors and apply brightness
                    let r = (r1 as f32 * (1.0 - segment_progress) + r2 as f32 * segment_progress) as u8;
                    let g = (g1 as f32 * (1.0 - segment_progress) + g2 as f32 * segment_progress) as u8;
                    let b = (b1 as f32 * (1.0 - segment_progress) + b2 as f32 * segment_progress) as u8;
                    
                    // Apply brightness scaling
                    let [r, g, b] = self.apply_brightness([r, g, b]);
                    
                    // Map position to actual pixel on display (2 pixels thick)
                    if pos < width as usize {
                        // Top border
                        canvas.set_pixel(pos, 0, r, g, b);
                        canvas.set_pixel(pos, 1, r, g, b); // Second row
                    } else if pos < (width as usize) * 2 {
                        // Bottom border
                        canvas.set_pixel(pos - width as usize, (height - 1) as usize, r, g, b);
                        canvas.set_pixel(pos - width as usize, (height - 2) as usize, r, g, b); // Second row
                    } else if pos < (width as usize) * 2 + (height as usize) - 2 {
                        // Left border (excluding corners)
                        canvas.set_pixel(0, pos - (width as usize) * 2 + 1, r, g, b);
                        canvas.set_pixel(1, pos - (width as usize) * 2 + 1, r, g, b); // Second column
                    } else {
                        // Right border (excluding corners)
                        canvas.set_pixel((width - 1) as usize, 
                                       pos - (width as usize) * 2 - (height as usize) + 2 + 1, 
                                       r, g, b);
                        canvas.set_pixel((width - 2) as usize, 
                                       pos - (width as usize) * 2 - (height as usize) + 2 + 1, 
                                       r, g, b); // Second column
                    }
                }
            }
        }
    }

    pub fn update_display(&mut self, position: i32) {
        let mut canvas = self.canvas.take().expect("Canvas missing");
        canvas.fill(0, 0, 0);  // Always clear the canvas
        
        let current = self.get_current_content().clone();
        
        // Extract text content from the current content
        #[allow(irrefutable_let_patterns)]
        if let ContentDetails::Text(text_content) = &current.content.data {
            // Apply brightness scaling to the text color
            let [r, g, b] = self.apply_brightness(text_content.color);
            let default_text_style = MonoTextStyle::new(&FONT_10X20_LATIN1, Rgb888::new(r, g, b));
            
            self.text_width = self.calculate_text_width(&text_content.text, &default_text_style);
            
            // Dynamic vertical centering calculation
            let font_height = 20;
            let baseline_adjustment = 5;
            let vertical_position = (self.display_height / 2) + (font_height / 2) - baseline_adjustment;
            
            // Create the embedded graphics canvas wrapper
            let mut eg_canvas = EmbeddedGraphicsCanvas::new(&mut canvas);
            
            if let Some(segments) = &text_content.text_segments {
                if !segments.is_empty() {
                    // Render each text segment with its specific formatting
                    let mut x_pos = if text_content.scroll {
                        position
                    } else {
                        (self.display_width - self.text_width) / 2
                    };
                    
                    // Collect formatting data to apply after text rendering
                    let mut formatting_effects = Vec::new();
                    
                    // First pass: render all text
                    for segment in segments {
                        // Apply brightness scaling to segment color
                        // Use the segment color if specified, otherwise fall back to the default text color
                        let segment_color = segment.color.unwrap_or(text_content.color);
                        let [sr, sg, sb] = self.apply_brightness(segment_color);
                        
                        // Always use the same font for consistent sizing
                        let font = &FONT_10X20_LATIN1;
                        let segment_style = MonoTextStyle::new(font, Rgb888::new(sr, sg, sb));
                        
                        // Extract the segment text from the main content text
                        // Convert the full text to a vector of characters for safe indexing
                        let chars: Vec<char> = text_content.text.chars().collect();
                        
                        // Make sure indices are within bounds
                        let start = segment.start.min(chars.len());
                        let end = segment.end.min(chars.len());
                        
                        if start < end {
                            // Get the text for this segment
                            let segment_text: String = chars[start..end].iter().collect();
                            
                            // Calculate segment width consistently
                            let segment_width = (end - start) as i32 * 10;
                            
                            // Check for bold formatting if formatting exists
                            let has_bold = segment.formatting.as_ref().map_or(false, |fmt| fmt.bold);
                            
                            // Render the text
                            if has_bold {
                                // Draw text twice with a 1px offset to create a bold effect
                                Text::new(&segment_text, Point::new(x_pos + 1, vertical_position), segment_style)
                                    .draw(&mut eg_canvas)
                                    .unwrap();
                            }
                            Text::new(&segment_text, Point::new(x_pos, vertical_position), segment_style)
                                .draw(&mut eg_canvas)
                                .unwrap();
                            
                            // Store formatting data for later processing
                            let has_underline = segment.formatting.as_ref().map_or(false, |fmt| fmt.underline);
                            let has_strikethrough = segment.formatting.as_ref().map_or(false, |fmt| fmt.strikethrough);
                            
                            if has_underline || has_strikethrough {
                                formatting_effects.push((
                                    x_pos,
                                    segment_width,
                                    [sr, sg, sb],  // Store segment color for reference
                                    has_underline,
                                    has_strikethrough
                                ));
                            }
                            
                            // Move x position for next segment
                            x_pos += segment_width;
                        }
                    }
                    
                    // Drop eg_canvas to release mutable borrow on canvas
                    drop(eg_canvas);
                    
                    // Second pass: apply underline and strikethrough effects directly to canvas
                    for (x_pos, width, [r, g, b], is_underline, is_strikethrough) in formatting_effects {
                        if is_underline {
                            // Draw line 1px below text
                            let underline_y = vertical_position + 3;
                            for i in 0..width {
                                canvas.set_pixel((x_pos + i) as usize, underline_y as usize, r, g, b);
                            }
                        }
                        
                        if is_strikethrough {
                            // Get strikethrough color - red for white text, white for everything else
                            let [strike_r, strike_g, strike_b] = get_smooth_strikethrough_color(r, g, b);
                            let strike_color_with_brightness = self.apply_brightness([strike_r, strike_g, strike_b]);
                            
                            // Draw line through text - two pixels high for better visibility
                            let strike_y1 = vertical_position - font_height/5; 
                            let strike_y2 = strike_y1 - 1; // Second line one pixel above
                            
                            for i in 0..width {
                                // Draw two pixels in height
                                canvas.set_pixel((x_pos + i) as usize, strike_y1 as usize, strike_color_with_brightness[0], strike_color_with_brightness[1], strike_color_with_brightness[2]);
                                canvas.set_pixel((x_pos + i) as usize, strike_y2 as usize, strike_color_with_brightness[0], strike_color_with_brightness[1], strike_color_with_brightness[2]);
                            }
                        }
                    }
                }
                else {
                    // Fallback to regular text rendering if segments list is empty
                    self.render_unsegmented_text(&mut eg_canvas, text_content, position, vertical_position, &default_text_style);
                    // Drop eg_canvas to release borrow
                    drop(eg_canvas);
                }
            }
            else {
                // No segments defined, render normal text
                self.render_unsegmented_text(&mut eg_canvas, text_content, position, vertical_position, &default_text_style);
                // Drop eg_canvas to release borrow
                drop(eg_canvas);
            }
        }
        
        // Draw border effect with brightness scaling
        if let Some(effect) = &current.border_effect {
            if effect != &BorderEffect::None {
                self.draw_border(&mut canvas, effect);
            }
        }
        
        // Use our driver to update the canvas
        let updated_canvas = self.driver.update_canvas(canvas);
        self.canvas = Some(updated_canvas);
    }
    
    // Modify render_unsegmented_text to accept TextContent
    fn render_unsegmented_text(
        &self, 
        eg_canvas: &mut EmbeddedGraphicsCanvas, 
        content: &TextContent,
        position: i32,
        vertical_position: i32,
        text_style: &MonoTextStyle<Rgb888>
    ) {
        if content.scroll {
            Text::new(&content.text, Point::new(position, vertical_position), *text_style)
                .draw(eg_canvas)
                .unwrap();
        } else {
            let x = (self.display_width - self.text_width) / 2;
            Text::new(&content.text, Point::new(x, vertical_position), *text_style)
                .draw(eg_canvas)
                .unwrap();
        }
    }

    // Update the method to apply brightness to a color array
    fn apply_brightness(&self, color: [u8; 3]) -> [u8; 3] {
        let brightness_scale = self.config.get_effective_brightness();
        [
            (color[0] as f32 * brightness_scale) as u8,
            (color[1] as f32 * brightness_scale) as u8,
            (color[2] as f32 * brightness_scale) as u8,
        ]
    }

    // Add a method to check if the playlist is empty
    #[allow(dead_code)]
    pub fn is_playlist_empty(&self) -> bool {
        self.playlist.items.is_empty()
    }

    // Add a method to get the current brightness
    pub fn get_brightness(&self) -> u8 {
        self.config.user_brightness
    }

    pub fn shutdown(&mut self) {
        info!("Shutting down display manager");
        
        // First clear the canvas if we have one
        if let Some(mut canvas) = self.canvas.take() {
            canvas.fill(0, 0, 0); // Clear to black
            // Put the cleared canvas back
            self.canvas = Some(canvas);
            // Update the display one more time to show the black screen
            self.update_display(0);
        }
        
        // Then shut down the driver
        self.driver.shutdown();
    }

    // Update the method to set brightness without reinitializing
    pub fn set_brightness(&mut self, brightness: u8) {
        let brightness = brightness.clamp(0, 100);
        info!("Updating display brightness: {}", brightness);
        
        // Update the brightness in the config instead of the playlist
        self.config.user_brightness = brightness;
    }

    // Handle content preview with scroll position preservation where possible
    pub fn enter_preview_mode(&mut self, content: DisplayContent) {
        let already_in_preview = self.preview_mode;
        self.preview_mode = true;
        self.last_preview_ping = Instant::now();
        
        if already_in_preview && self.preview_content.is_some() {
            let previous_content = self.preview_content.as_ref().unwrap();
            
            // Extract text content from both previous and new content
            #[allow(irrefutable_let_patterns)]
            if let (ContentDetails::Text(prev_text), ContentDetails::Text(new_text)) = 
                (&previous_content.content.data, &content.content.data) {
                
                // Determine if we need to reset scroll position
                let should_reset = 
                    // Reset if scroll mode changes
                    prev_text.scroll != new_text.scroll ||
                    // Reset if empty text becomes non-empty or vice versa
                    (prev_text.text.is_empty() != new_text.text.is_empty()) ||
                    // Reset if text length changes dramatically (by more than 50%)
                    (!prev_text.text.is_empty() && !new_text.text.is_empty() && 
                     (prev_text.text.len() as f32 / new_text.text.len() as f32 > 1.5 || 
                      new_text.text.len() as f32 / prev_text.text.len() as f32 > 1.5));
                        
                if !should_reset {
                    // Silent update - preserve scroll position
                    self.preview_content = Some(content);
                    return;
                }
                
                debug!("Resetting scroll position due to significant content change");
            } else {
                // Content type changed, we definitely need to reset
                debug!("Resetting scroll position due to content type change");
            }
        }
        else if !already_in_preview {
            // Only log once when first entering preview mode
            info!("Entering preview mode");
        }
        
        // Reset everything for new preview or significant content change
        self.preview_content = Some(content);
        self.scroll_position = self.display_width;
        self.completed_scrolls = 0;
        self.current_repeat = 0;
        self.last_transition = Instant::now();
    }

    // Exit preview mode and return to normal playlist playback
    pub fn exit_preview_mode(&mut self) {
        if self.preview_mode {
            info!("Exiting preview mode");
            self.preview_mode = false;
            self.preview_content = None;
            
            // Reset display state for normal playback
            self.scroll_position = self.display_width;
            self.completed_scrolls = 0;
            self.current_repeat = 0;
            self.last_transition = Instant::now();
        }
    }

    // Update ping time to keep preview mode active
    pub fn ping_preview_mode(&mut self) -> bool {
        if self.preview_mode {
            self.last_preview_ping = Instant::now();
            debug!("Preview mode ping received");
            true
        } else {
            false
        }
    }

    // Check if preview mode has timed out from inactivity
    pub fn check_preview_timeout(&mut self, timeout_seconds: u64) -> bool {
        if self.preview_mode {
            let elapsed = self.last_preview_ping.elapsed().as_secs();
            if elapsed > timeout_seconds {
                info!("Preview mode timed out after {} seconds of inactivity", elapsed);
                self.exit_preview_mode();
                return true;
            }
        }
        false
    }

    // Check if preview mode is currently active
    pub fn is_in_preview_mode(&self) -> bool {
        self.preview_mode
    }
}

// Convert HSV (Hue, Saturation, Value) to RGB
// h: 0.0-1.0 (hue), s: 0.0-1.0 (saturation), v: 0.0-1.0 (value)
// Returns (r, g, b) as u8 values (0-255)
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

// Add this helper function to get the local IP address
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    
    // This is a common trick to get the local IP address
    // We don't actually send anything, just use it to determine the local interface
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            // Try to "connect" to a public IP (doesn't actually send anything)
            if socket.connect("8.8.8.8:80").is_ok() {
                // Get the local address the socket is bound to
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }
            None
        },
        Err(_) => None
    }
}

// Helper function to get color for strikethrough - updated for arrays
fn get_smooth_strikethrough_color(r: u8, g: u8, b: u8) -> [u8; 3] {
    // Check if we're in grayscale mode (R≈G≈B)
    let is_grayscale = (r as i16 - g as i16).abs() < 20 && 
                       (g as i16 - b as i16).abs() < 20 && 
                       (r as i16 - b as i16).abs() < 20;
    
    // For grayscale colors (white to gray to black), use red
    if is_grayscale {
        return [255, 0, 0];
    }
    
    // Check if the color is in the red family where G≈B
    let g_equals_b = (g as i16 - b as i16).abs() < 20;
    
    if g_equals_b && r > g + 30 {
        // This is a true red family color (red with equal G and B components)
        // Calculate the blend factor based on red's dominance
        
        // Use a ratio to determine how red-dominant the color is
        let red_ratio = r as f32 / (r as f32 + g as f32 + b as f32);
        
        // For bright red colors (like 255,0,0), we want white strikethrough
        // For pinkish colors (like 255,200,200), we want red strikethrough
        let blend_factor = ((red_ratio - 0.4) * 2.5).min(1.0).max(0.0);
        
        // Create a color between red and white (white for strong reds)
        let strike_r = 255;
        let strike_g = (blend_factor * 255.0) as u8;
        let strike_b = (blend_factor * 255.0) as u8;
        
        [strike_r, strike_g, strike_b]
    } 
    // Handle mid-gray to mid-red axis (like 139,139,139 to 139,0,0)
    else if g_equals_b && r > g {
        // Similar logic but for mid-range colors
        let red_ratio = r as f32 / (r as f32 + g as f32 + b as f32);
        
        // Use the same calculation as for bright reds
        let blend_factor = ((red_ratio - 0.4) * 2.5).min(1.0).max(0.0);
        
        let strike_r = 255;
        let strike_g = (blend_factor * 255.0) as u8;
        let strike_b = (blend_factor * 255.0) as u8;
        
        [strike_r, strike_g, strike_b]
    } else {
        // For all other colors, use white strikethrough
        [255, 255, 255]
    }
} 