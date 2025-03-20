use crate::models::{DisplayContent, Playlist, BorderEffect, ContentType};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    text::Text,
    pixelcolor::Rgb888,
    geometry::Point,
    Drawable,
};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use log::{info, error, debug};
use rand::Rng;
use crate::led_driver::{LedDriver, LedCanvas, create_driver};
use crate::embedded_graphics_support::EmbeddedGraphicsCanvas;
use crate::config::DisplayConfig;

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
}

impl DisplayManager {
    pub fn with_config(config: DisplayConfig) -> Self {
        // Get display dimensions
        let display_width = config.display_width();
        let display_height = config.display_height();
        
        info!("Initializing display: {}x{} (rows={}, cols={}, chain={}, parallel={})",
              display_width, display_height, config.rows, config.cols, 
              config.chain_length, config.parallel);
        
        // Initialize the driver
        let driver = create_driver(&config)
            .expect("Failed to initialize LED matrix driver");
        
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
            config,
        }
    }

    pub fn with_playlist_and_config(playlist: Playlist, config: DisplayConfig) -> Self {
        // Get dimensions
        let display_width = config.display_width();
        let display_height = config.display_height();
        
        info!("Initializing display with playlist: {}x{} (rows={}, cols={}, chain={}, parallel={})",
              display_width, display_height, config.rows, config.cols, 
              config.chain_length, config.parallel);
        
        // Initialize the driver
        let driver = create_driver(&config)
            .expect("Failed to initialize LED matrix driver");
        
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
            config,
        }
    }

    pub fn calculate_text_width(&self, text: &str, _style: &MonoTextStyle<Rgb888>) -> i32 {
        // FONT_10X20 is 10 pixels wide per character, add a small buffer
        (text.len() as i32) * 10 + 2
    }

    pub fn get_current_content(&self) -> &DisplayContent {
        if self.playlist.items.is_empty() {
            // Store the default message item
            static DEFAULT_ITEM: Lazy<DisplayContent> = Lazy::new(|| {
                // Get the local IP for a more helpful message
                let ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());
                
                DisplayContent {
                    content_type: ContentType::Text,
                    text: format!("LED Matrix Controller | Web interface: http://{}:3000 | Use web UI to configure display", ip),
                    scroll: true,
                    color: (0, 255, 0),  // Green color for visibility
                    speed: 30.0,         // Slower for better readability
                    duration: 0,
                    repeat_count: 0,     // Infinite repeat
                    border_effect: Some(BorderEffect::Pulse { colors: vec![(0, 255, 0), (0, 200, 0)] }), // Add a nice pulsing border
                    colored_segments: None,
                }
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
                    let (r1, g1, b1) = colors[segment_idx];
                    let (r2, g2, b2) = colors[next_segment_idx];
                    
                    // Interpolate colors
                    let r = (r1 as f32 * (1.0 - segment_progress) + r2 as f32 * segment_progress) as u8;
                    let g = (g1 as f32 * (1.0 - segment_progress) + g2 as f32 * segment_progress) as u8;
                    let b = (b1 as f32 * (1.0 - segment_progress) + b2 as f32 * segment_progress) as u8;
                    
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
        let (r, g, b) = current.color;
        let default_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(r, g, b));
        
        self.text_width = self.calculate_text_width(&current.text, &default_text_style);
        
        // Dynamic vertical centering calculation
        let font_height = 20;
        let baseline_adjustment = 5;
        let vertical_position = (self.display_height / 2) + (font_height / 2) - baseline_adjustment;
        
        // Create the embedded graphics canvas wrapper
        let mut eg_canvas = EmbeddedGraphicsCanvas::new(&mut canvas);
        
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
                            .draw(&mut eg_canvas)
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
                        .draw(&mut eg_canvas)
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
                    .draw(&mut eg_canvas)
                    .unwrap();
            }
        } else {
            // Render text with a single color
            if current.scroll {
                Text::new(&current.text, Point::new(position, vertical_position), default_text_style)
                    .draw(&mut eg_canvas)
                    .unwrap();
            } else {
                let x = (self.display_width - self.text_width) / 2;
                Text::new(&current.text, Point::new(x, vertical_position), default_text_style)
                    .draw(&mut eg_canvas)
                    .unwrap();
            }
        }
        
        // Draw border effect if one is specified
        if let Some(effect) = &current.border_effect {
            if effect != &BorderEffect::None {
                self.draw_border(&mut canvas, effect);
            }
        }
        
        // Use our driver to update the canvas
        let updated_canvas = self.driver.update_canvas(canvas);
        self.canvas = Some(updated_canvas);
    }

    pub fn reinitialize_with_brightness(&mut self, brightness: u8) {
        let brightness = brightness.clamp(0, 100);
        info!("Reinitializing display with brightness: {}", brightness);
        
        // Update our config
        let mut display_config = self.config.clone();
        display_config.led_brightness = brightness;
        
        // Create matrix config and log details
        info!("Reinitializing display: {}x{} (rows={}, cols={}, chain={}, parallel={}, brightness={})",
              display_config.display_width(), display_config.display_height(),
              display_config.rows, display_config.cols, 
              display_config.chain_length, display_config.parallel, brightness);
        
        // Initialize the driver
        match create_driver(&display_config) {
            Ok(driver) => {
                debug!("LED matrix driver reinitialized successfully");
                let mut driver_box = driver;
                let canvas = driver_box.take_canvas();
                
                self.driver = driver_box;
                self.canvas = canvas;
                
                // Update our configuration
                self.config = display_config;
                
                // Update dimensions
                self.display_width = self.config.display_width();
                self.display_height = self.config.display_height();
                self.scroll_position = self.display_width;
                
                // Update global brightness in the playlist
                self.playlist.brightness = brightness;
            },
            Err(e) => {
                error!("Failed to reinitialize driver: {}", e);
                // Continue using the existing driver rather than crashing
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