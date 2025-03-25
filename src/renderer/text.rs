use crate::models::{DisplayContent, ContentDetails, TextContent, TextSegment};
use crate::led_driver::LedCanvas;
use crate::embedded_graphics_support::EmbeddedGraphicsCanvas;
use crate::renderer::{Renderer, RenderContext};
use embedded_graphics::mono_font::iso_8859_1::FONT_10X20 as FONT_10X20_LATIN1;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::geometry::Point;
use embedded_graphics::Drawable;
use std::time::Instant;
use log::{debug};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct TextRenderer {
    /// The text content to render
    content: TextContent,
    
    /// Context with display properties
    ctx: RenderContext,
    
    /// Width of the text in pixels
    text_width: i32,
    
    /// Current scroll position
    scroll_position: i32,
    
    /// Counter for completed scroll cycles
    completed_scrolls: u32,
    
    /// Timing accumulator for scroll animation
    accumulated_time: f32,
    
    /// Target number of repeats (None for duration-based)
    repeat_count: Option<u32>,
    
    /// Duration-based timing
    duration: Option<u64>,
    
    /// Timestamp when rendering started
    start_time: Instant,
    
    /// Last reported cycle (to avoid duplicate logging)
    last_reported_cycle: AtomicU32,
}

impl Renderer for TextRenderer {
    fn new(content: &DisplayContent, ctx: RenderContext) -> Self {
        // Extract the text content from the display content
        let text_content = match &content.content.data {
            ContentDetails::Text(tc) => tc.clone(),
            #[allow(unreachable_patterns)]
            _ => panic!("Expected text content")
        };
        
        // Create text renderer with clone of ctx
        let ctx_clone = ctx.clone();
        let mut renderer = Self {
            content: text_content,
            ctx: ctx_clone,
            text_width: 0, // Will calculate on first render
            scroll_position: ctx.display_width,
            completed_scrolls: 0,
            accumulated_time: 0.0,
            repeat_count: content.repeat_count,
            duration: content.duration,
            start_time: Instant::now(),
            last_reported_cycle: AtomicU32::new(0),
        };
        
        // Pre-calculate text width
        renderer.calculate_text_width();
        
        // Log the configuration to help diagnose issues
        debug!("TextRenderer::new - text: '{}', scroll: {}, duration: {:?}, repeat_count: {:?}",
               renderer.content.text, 
               renderer.content.scroll,
               renderer.duration,
               renderer.repeat_count);
        
        renderer
    }
    
    fn update(&mut self, dt: f32) {
        if self.content.scroll {
            self.accumulated_time += dt;
            let pixels_to_move = (self.accumulated_time * self.content.speed) as i32;
            
            if pixels_to_move > 0 {
                self.scroll_position -= pixels_to_move;
                self.accumulated_time = 0.0;
                
                // Reset position when text is off screen
                if self.scroll_position < -self.text_width {
                    self.scroll_position = self.ctx.display_width;
                    self.completed_scrolls += 1;
                }
            }
        } 
        // For duration-based content, track elapsed time
        else if let Some(_) = self.duration {
            // Calculate elapsed time in seconds
            let elapsed = Instant::now().duration_since(self.start_time).as_secs();
            // Track elapsed time for is_complete() functionality
            self.last_reported_cycle.store(elapsed as u32, Ordering::SeqCst);
        }
    }
    
    fn render(&self, canvas: &mut Box<dyn LedCanvas>) {
        // Create embedded graphics wrapper
        let mut eg_canvas = EmbeddedGraphicsCanvas::new(canvas);
        
        // Get the vertical position for text
        let font_height = 20; // Height of FONT_10X20_LATIN1
        let vertical_position = self.ctx.calculate_centered_text_position(font_height);
        
        // Apply brightness scaling to the text color
        let [r, g, b] = self.ctx.apply_brightness(self.content.color);
        let text_style = MonoTextStyle::new(&FONT_10X20_LATIN1, Rgb888::new(r, g, b));
        
        if let Some(segments) = &self.content.text_segments {
            if !segments.is_empty() {
                self.render_segmented_text(&mut eg_canvas, segments, vertical_position);
            } else {
                self.render_simple_text(&mut eg_canvas, vertical_position, &text_style);
            }
        } else {
            self.render_simple_text(&mut eg_canvas, vertical_position, &text_style);
        }
    }
    
    fn is_complete(&self) -> bool {
        // For duration-based content
        if let Some(duration) = self.duration {
            return Instant::now().duration_since(self.start_time).as_secs() >= duration;
        }
        
        // For repeat-count based content
        if let Some(repeat_count) = self.repeat_count {
            if repeat_count == 0 {
                return false; // Infinite repeat
            }
            return self.completed_scrolls >= repeat_count;
        }
        
        false // Default case
    }
    
    fn reset(&mut self) {
        self.scroll_position = self.ctx.display_width;
        self.completed_scrolls = 0;
        self.accumulated_time = 0.0;
        self.start_time = Instant::now();
        self.last_reported_cycle.store(0, Ordering::SeqCst);
    }
    
    fn update_context(&mut self, ctx: RenderContext) {
        // Update the context without changing animation state
        self.ctx = ctx;
    }
    
    fn update_content(&mut self, content: &DisplayContent) {
        // Extract the new text content
        let new_text_content = match &content.content.data {
            ContentDetails::Text(tc) => tc.clone(),
            #[allow(unreachable_patterns)]
            _ => panic!("Expected text content")
        };
        
        // Track if we need to recalculate width
        let text_changed = self.content.text != new_text_content.text;
        
        // Update content properties
        self.content = new_text_content;
        self.repeat_count = content.repeat_count;
        self.duration = content.duration;
        
        // Only recalculate width if text changed
        if text_changed {
            self.calculate_text_width();
            
            // Don't reset scroll position completely, but ensure it's visible
            // if currently off-screen
            if self.content.scroll && self.scroll_position < -self.text_width {
                // Position text just off screen to the right
                self.scroll_position = self.ctx.display_width;
            }
        }
        
        // Log that we're preserving animation state
        debug!("Updated TextRenderer content while preserving animation state");
    }
}

impl TextRenderer {
    // Calculate text width based on character count
    fn calculate_text_width(&mut self) {
        self.text_width = (self.content.text.chars().count() as i32) * 10 + 2;
    }
    
    // Render simple (unsegmented) text
    fn render_simple_text(&self, canvas: &mut EmbeddedGraphicsCanvas, y_pos: i32, style: &MonoTextStyle<Rgb888>) {
        if self.content.scroll {
            Text::new(&self.content.text, Point::new(self.scroll_position, y_pos), *style)
                .draw(canvas)
                .unwrap();
        } else {
            let x = (self.ctx.display_width - self.text_width) / 2;
            Text::new(&self.content.text, Point::new(x, y_pos), *style)
                .draw(canvas)
                .unwrap();
        }
    }
    
    // Render segmented text with formatting
    fn render_segmented_text(&self, canvas: &mut EmbeddedGraphicsCanvas, segments: &[TextSegment], y_pos: i32) {
        // Starting X position depends on scroll mode
        let x_start = if self.content.scroll {
            self.scroll_position
        } else {
            (self.ctx.display_width - self.text_width) / 2
        };
        
        // Collect formatting data to apply after text rendering
        let mut formatting_effects = Vec::new();
        
        // Convert the full text to a vector of characters for safe indexing
        let chars: Vec<char> = self.content.text.chars().collect();
        
        // First pass: render all text segments
        for segment in segments {
            // Apply brightness scaling to segment color
            // Use the segment color if specified, otherwise fall back to the default text color
            let segment_color = segment.color.unwrap_or(self.content.color);
            let [sr, sg, sb] = self.ctx.apply_brightness(segment_color);
            
            // Create text style for this segment
            let font = &FONT_10X20_LATIN1;
            let segment_style = MonoTextStyle::new(font, Rgb888::new(sr, sg, sb));
            
            // Make sure indices are within bounds
            let start = segment.start.min(chars.len());
            let end = segment.end.min(chars.len());
            
            if start < end {
                // Get the text for this segment
                let segment_text: String = chars[start..end].iter().collect();
                
                // Calculate segment width and position
                let segment_width = (end - start) as i32 * 10;
                let x_pos = x_start + (start as i32 * 10);
                
                // Check for bold formatting
                let has_bold = segment.formatting.as_ref().map_or(false, |fmt| fmt.bold);
                
                // Render the text
                if has_bold {
                    // Draw text twice with a 1px offset to create a bold effect
                    Text::new(&segment_text, Point::new(x_pos + 1, y_pos), segment_style)
                        .draw(canvas)
                        .unwrap();
                }
                
                Text::new(&segment_text, Point::new(x_pos, y_pos), segment_style)
                    .draw(canvas)
                    .unwrap();
                
                // Store formatting data for second pass
                let has_underline = segment.formatting.as_ref().map_or(false, |fmt| fmt.underline);
                let has_strikethrough = segment.formatting.as_ref().map_or(false, |fmt| fmt.strikethrough);
                
                if has_underline || has_strikethrough {
                    formatting_effects.push((
                        x_pos,
                        segment_width,
                        [sr, sg, sb],
                        has_underline,
                        has_strikethrough
                    ));
                }
            }
        }
        
        // Second pass: apply underline and strikethrough effects
        for (x_pos, width, [r, g, b], is_underline, is_strikethrough) in formatting_effects {
            self.apply_text_effects(canvas, x_pos, width, y_pos, [r, g, b], is_underline, is_strikethrough);
        }
    }
    
    // Apply underline and strikethrough effects
    fn apply_text_effects(
        &self, 
        eg_canvas: &mut EmbeddedGraphicsCanvas,
        x_pos: i32, 
        width: i32, 
        y_pos: i32,
        [r, g, b]: [u8; 3],
        is_underline: bool,
        is_strikethrough: bool
    ) {
        if is_underline {
            // Draw line 3px below text baseline using embedded graphics primitives
            let underline_y = y_pos + 3;
            
            // Get the underlying canvas from EmbeddedGraphicsCanvas
            let canvas = eg_canvas.inner_mut();
            
            for i in 0..width {
                canvas.set_pixel((x_pos + i) as usize, underline_y as usize, r, g, b);
            }
        }
        
        if is_strikethrough {
            // Get contrasting color for strikethrough
            let [strike_r, strike_g, strike_b] = self.get_strikethrough_color(r, g, b);
            
            // Draw line through text center (font_height = 20)
            let strike_y1 = y_pos - 5; 
            let strike_y2 = strike_y1 - 1; // Second line one pixel above
            
            // Get the underlying canvas
            let canvas = eg_canvas.inner_mut();
            
            for i in 0..width {
                // Draw two pixels in height for better visibility
                canvas.set_pixel((x_pos + i) as usize, strike_y1 as usize, strike_r, strike_g, strike_b);
                canvas.set_pixel((x_pos + i) as usize, strike_y2 as usize, strike_r, strike_g, strike_b);
            }
        }
    }
    
    // Helper to get appropriate strikethrough color
    fn get_strikethrough_color(&self, r: u8, g: u8, b: u8) -> [u8; 3] {
        // Check if we're in grayscale mode (R≈G≈B)
        let is_grayscale = (r as i16 - g as i16).abs() < 20 && 
                           (g as i16 - b as i16).abs() < 20 && 
                           (r as i16 - b as i16).abs() < 20;
        
        // For grayscale colors, use red
        if is_grayscale {
            return self.ctx.apply_brightness([255, 0, 0]);
        }
        
        // For red family colors
        let g_equals_b = (g as i16 - b as i16).abs() < 20;
        if g_equals_b && r > g + 30 {
            let red_ratio = r as f32 / (r as f32 + g as f32 + b as f32);
            let blend_factor = ((red_ratio - 0.4) * 2.5).min(1.0).max(0.0);
            
            let strike_r = 255;
            let strike_g = (blend_factor * 255.0) as u8;
            let strike_b = (blend_factor * 255.0) as u8;
            
            return self.ctx.apply_brightness([strike_r, strike_g, strike_b]);
        }
        
        // Default to white for all other colors
        self.ctx.apply_brightness([255, 255, 255])
    }
} 