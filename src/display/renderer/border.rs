use crate::display::driver::LedCanvas;
use crate::display::renderer::{RenderContext, Renderer};
use crate::models::border_effects::BorderEffect;
use crate::models::playlist::PlayListItem;
use std::time::Instant;

pub struct BorderRenderer {
    /// The border effect to render
    effect: BorderEffect,

    /// Context with display properties
    ctx: RenderContext,

    /// Animation state for animated borders (0.0-∞)
    animation_state: f32,

    /// Timestamp when rendering started
    start_time: Instant,
}

impl Renderer for BorderRenderer {
    fn new(content: &PlayListItem, ctx: RenderContext) -> Self {
        // Extract the border effect from the content, or use None if not specified
        let effect = content.border_effect.clone().unwrap_or(BorderEffect::None);

        Self {
            effect,
            ctx: ctx.clone(), // Clone to avoid move issues
            animation_state: 0.0,
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, dt: f32) {
        // Update animation state for animated borders
        match &self.effect {
            BorderEffect::None => {} // No state to update
            _ => {
                // Accumulate time for continuous animation
                self.animation_state += dt;
            }
        }
    }

    fn render(&self, canvas: &mut Box<dyn LedCanvas>) {
        match &self.effect {
            BorderEffect::None => {
                // No border to render
            }
            BorderEffect::Rainbow => {
                self.render_rainbow_border(canvas);
            }
            BorderEffect::Pulse { colors } => {
                self.render_pulse_border(canvas, colors);
            }
            BorderEffect::Sparkle { colors } => {
                self.render_sparkle_border(canvas, colors);
            }
            BorderEffect::Gradient { colors } => {
                self.render_gradient_border(canvas, colors);
            }
        }
    }

    // Border renderers don't determine content completion
    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        self.animation_state = 0.0;
        self.start_time = Instant::now();
    }

    fn update_context(&mut self, ctx: RenderContext) {
        // Update the context without changing animation state
        self.ctx = ctx;
    }

    fn update_content(&mut self, content: &PlayListItem) {
        // Get the new border effect (or None)
        let new_effect = content.border_effect.clone().unwrap_or(BorderEffect::None);

        // Only update the effect, preserving animation state
        self.effect = new_effect;
    }
}

impl BorderRenderer {
    // Render a rainbow border effect
    fn render_rainbow_border(&self, canvas: &mut Box<dyn LedCanvas>) {
        let height = self.ctx.display_height;
        let width = self.ctx.display_width;

        // Draw top and bottom rainbow
        for i in 0..width {
            let hue = (i as f32 / width as f32 + self.animation_state) % 1.0;
            let (r, g, b) = self.hsv_to_rgb(hue, 1.0, 1.0);
            let [r, g, b] = self.ctx.apply_brightness([r, g, b]);

            // Top border (2 pixels thick)
            canvas.set_pixel(i as usize, 0, r, g, b);
            canvas.set_pixel(i as usize, 1, r, g, b);

            // Bottom border (2 pixels thick)
            canvas.set_pixel(i as usize, (height - 1) as usize, r, g, b);
            canvas.set_pixel(i as usize, (height - 2) as usize, r, g, b);
        }

        // Draw left and right rainbow
        for i in 0..height {
            let hue = (i as f32 / height as f32 + self.animation_state) % 1.0;
            let (r, g, b) = self.hsv_to_rgb(hue, 1.0, 1.0);
            let [r, g, b] = self.ctx.apply_brightness([r, g, b]);

            // Left border (2 pixels thick)
            canvas.set_pixel(0, i as usize, r, g, b);
            canvas.set_pixel(1, i as usize, r, g, b);

            // Right border (2 pixels thick)
            canvas.set_pixel((width - 1) as usize, i as usize, r, g, b);
            canvas.set_pixel((width - 2) as usize, i as usize, r, g, b);
        }
    }

    // Render a pulsing border effect
    fn render_pulse_border(&self, canvas: &mut Box<dyn LedCanvas>, colors: &[[u8; 3]]) {
        let _height = self.ctx.display_height;
        let _width = self.ctx.display_width;

        // Handle empty colors case
        if colors.is_empty() {
            return;
        }

        // Speed up the animation by adjusting the time factor
        let adjusted_time = self.animation_state * 0.7;

        // Each color cycle: 2 seconds (1s fade in, 1s fade out)
        let seconds_per_color = 2.0;
        let total_cycle = seconds_per_color * colors.len() as f32;

        // Figure out which color we're currently displaying
        let current_position = adjusted_time % total_cycle;
        let color_index = (current_position / seconds_per_color) as usize;

        // Safety check for array bounds
        if color_index >= colors.len() {
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
        let [r, g, b] = colors[color_index];
        let pre_scaled = [
            (r as f32 * effect_brightness) as u8,
            (g as f32 * effect_brightness) as u8,
            (b as f32 * effect_brightness) as u8,
        ];

        // Apply user brightness scaling
        let [r, g, b] = self.ctx.apply_brightness(pre_scaled);

        // Draw the border (2 pixels thick)
        self.draw_solid_border(canvas, r, g, b);
    }

    // Render a sparkling border effect
    fn render_sparkle_border(&self, canvas: &mut Box<dyn LedCanvas>, colors: &[[u8; 3]]) {
        let height = self.ctx.display_height;
        let width = self.ctx.display_width;

        // If no colors provided, don't render anything
        if colors.is_empty() {
            return;
        }

        // Create a new random generator each time - in a real implementation,
        // you might want to store this as a field for better performance
        let mut rng = rand::thread_rng();

        // Create sparkles based on animation state - increase count for thicker border
        for _ in 0..30 {
            // Increased from 20 to provide more density for 2-pixel border
            // Randomly select one of the available colors and apply brightness
            let color_index = rand::Rng::gen_range(&mut rng, 0..colors.len());
            let [r, g, b] = self.ctx.apply_brightness(colors[color_index]);

            // Random position along the border
            let pos = rand::Rng::gen_range(&mut rng, 0..2 * (width + height - 2));
            let inner = rand::Rng::gen_bool(&mut rng, 0.5); // 50% chance for inner or outer pixel

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
    }

    // Render a gradient border effect
    fn render_gradient_border(&self, canvas: &mut Box<dyn LedCanvas>, colors: &[[u8; 3]]) {
        let height = self.ctx.display_height;
        let width = self.ctx.display_width;

        if colors.is_empty() {
            return;
        }

        // Use at least 2 colors for gradient
        let colors = if colors.len() == 1 {
            vec![colors[0], colors[0]]
        } else {
            colors.to_vec()
        };

        let segments = colors.len();
        let perimeter = 2 * ((width as usize) + (height as usize) - 2);
        let segment_length = perimeter / segments;

        // Calculate offset for animation
        let offset = (self.animation_state * perimeter as f32) as usize;

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
            let [r, g, b] = self.ctx.apply_brightness([r, g, b]);

            // Map position to actual pixel on display (2 pixels thick)
            if pos < width as usize {
                // Top border
                canvas.set_pixel(pos, 0, r, g, b);
                canvas.set_pixel(pos, 1, r, g, b); // Second row
            } else if pos < (width as usize) * 2 {
                // Bottom border
                canvas.set_pixel(pos - width as usize, (height - 1) as usize, r, g, b);
                canvas.set_pixel(pos - width as usize, (height - 2) as usize, r, g, b);
            // Second row
            } else if pos < (width as usize) * 2 + (height as usize) - 2 {
                // Left border (excluding corners)
                canvas.set_pixel(0, pos - (width as usize) * 2 + 1, r, g, b);
                canvas.set_pixel(1, pos - (width as usize) * 2 + 1, r, g, b); // Second column
            } else {
                // Right border (excluding corners)
                canvas.set_pixel(
                    (width - 1) as usize,
                    pos - (width as usize) * 2 - (height as usize) + 2 + 1,
                    r,
                    g,
                    b,
                );
                canvas.set_pixel(
                    (width - 2) as usize,
                    pos - (width as usize) * 2 - (height as usize) + 2 + 1,
                    r,
                    g,
                    b,
                ); // Second column
            }
        }
    }

    // Helper to draw a solid border with the given color
    fn draw_solid_border(&self, canvas: &mut Box<dyn LedCanvas>, r: u8, g: u8, b: u8) {
        let height = self.ctx.display_height;
        let width = self.ctx.display_width;

        // Draw top and bottom borders
        for i in 0..width {
            // Top border (2 pixels thick)
            canvas.set_pixel(i as usize, 0, r, g, b);
            canvas.set_pixel(i as usize, 1, r, g, b);

            // Bottom border (2 pixels thick)
            canvas.set_pixel(i as usize, (height - 1) as usize, r, g, b);
            canvas.set_pixel(i as usize, (height - 2) as usize, r, g, b);
        }

        // Draw left and right borders
        for i in 0..height {
            // Left border (2 pixels thick)
            canvas.set_pixel(0, i as usize, r, g, b);
            canvas.set_pixel(1, i as usize, r, g, b);

            // Right border (2 pixels thick)
            canvas.set_pixel((width - 1) as usize, i as usize, r, g, b);
            canvas.set_pixel((width - 2) as usize, i as usize, r, g, b);
        }
    }

    // Convert HSV to RGB
    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> (u8, u8, u8) {
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
}
