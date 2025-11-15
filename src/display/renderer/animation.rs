use crate::display::driver::LedCanvas;
use crate::display::renderer::{RenderContext, Renderer};
use crate::models::animation::AnimationContent;
use crate::models::content::ContentDetails;
use crate::models::playlist::PlayListItem;
use std::f32::consts::TAU;
use std::time::Instant;

pub struct AnimationRenderer {
    content: AnimationContent,
    ctx: RenderContext,
    elapsed: f32,
    duration: Option<u64>,
    start_time: Instant,
}

impl Renderer for AnimationRenderer {
    fn new(content: &PlayListItem, ctx: RenderContext) -> Self {
        let animation_content = match &content.content.data {
            ContentDetails::Animation(ac) => ac.clone(),
            #[allow(unreachable_patterns)]
            _ => panic!("Expected animation content"),
        };

        Self {
            content: animation_content,
            ctx,
            elapsed: 0.0,
            duration: content.duration,
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    fn render(&self, canvas: &mut Box<dyn LedCanvas>) {
        match &self.content {
            AnimationContent::Pulse { colors, cycle_ms } => {
                self.render_pulse(canvas, colors, *cycle_ms as f32 / 1000.0);
            }
            AnimationContent::PaletteWave {
                colors,
                cycle_ms,
                wave_count,
            } => {
                self.render_palette_wave(canvas, colors, *cycle_ms as f32 / 1000.0, *wave_count);
            }
            AnimationContent::DualPulse {
                colors,
                cycle_ms,
                phase_offset,
            } => {
                self.render_dual_pulse(canvas, colors, *cycle_ms as f32 / 1000.0, *phase_offset);
            }
            AnimationContent::ColorFade {
                colors,
                drift_speed,
            } => {
                self.render_color_fade(canvas, colors, *drift_speed);
            }
            AnimationContent::Strobe {
                colors,
                flash_ms,
                fade_ms,
                randomize,
                randomization_factor,
            } => {
                self.render_strobe(
                    canvas,
                    colors,
                    *flash_ms,
                    *fade_ms,
                    *randomize,
                    *randomization_factor,
                );
            }
            AnimationContent::Sparkle {
                colors,
                density,
                twinkle_ms,
            } => {
                self.render_sparkle(canvas, colors, *density, *twinkle_ms);
            }
            AnimationContent::MosaicTwinkle {
                colors,
                tile_size,
                flow_speed,
                border_size,
                border_color,
            } => {
                self.render_mosaic_twinkle(
                    canvas,
                    colors,
                    *tile_size,
                    *flow_speed,
                    *border_size,
                    *border_color,
                );
            }
            AnimationContent::Plasma {
                colors,
                flow_speed,
                noise_scale,
            } => {
                self.render_plasma(canvas, colors, *flow_speed, *noise_scale);
            }
        }
    }

    fn is_complete(&self) -> bool {
        if let Some(duration) = self.duration {
            return Instant::now().duration_since(self.start_time).as_secs() >= duration;
        }
        false
    }

    fn reset(&mut self) {
        self.elapsed = 0.0;
        self.start_time = Instant::now();
    }

    fn update_context(&mut self, ctx: RenderContext) {
        self.ctx = ctx;
    }

    fn update_content(&mut self, content: &PlayListItem) {
        if let ContentDetails::Animation(animation_content) = &content.content.data {
            self.content = animation_content.clone();
            self.duration = content.duration;
        }
    }
}

impl AnimationRenderer {
    fn width(&self) -> usize {
        self.ctx.display_width as usize
    }

    fn height(&self) -> usize {
        self.ctx.display_height as usize
    }

    fn fill_canvas(&self, canvas: &mut Box<dyn LedCanvas>, color: [u8; 3]) {
        let [r, g, b] = self.ctx.apply_brightness(color);
        canvas.fill(r, g, b);
    }

    fn render_pulse(&self, canvas: &mut Box<dyn LedCanvas>, colors: &[[u8; 3]], cycle_s: f32) {
        if colors.is_empty() {
            return;
        }
        let progress = self.loop_progress(cycle_s);
        let color = self.sample_palette(colors, progress);
        let brightness = self.triangle_wave(progress);
        let scaled = Self::scale_color(color, brightness);
        self.fill_canvas(canvas, scaled);
    }

    fn render_dual_pulse(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        cycle_s: f32,
        phase_offset: f32,
    ) {
        if colors.is_empty() {
            return;
        }
        let progress = self.loop_progress(cycle_s);
        let second = (progress + phase_offset).fract();
        let brightness =
            (self.triangle_wave(progress) + self.triangle_wave(second)).clamp(0.0, 2.0) * 0.5;
        let color = self.sample_palette(colors, progress);
        let scaled = Self::scale_color(color, brightness);
        self.fill_canvas(canvas, scaled);
    }

    fn render_palette_wave(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        cycle_s: f32,
        wave_count: u8,
    ) {
        if colors.is_empty() {
            return;
        }

        let wave_count = wave_count.max(1) as f32;
        let offset = self.loop_progress(cycle_s);

        let width = self.width();
        let height = self.height();

        for y in 0..height {
            for x in 0..width {
                let norm_x = x as f32 / width as f32;
                let norm_y = y as f32 / height as f32;
                let base = (norm_x + norm_y * 0.25 + offset).fract();
                let wave = (base * wave_count).fract();
                let brightness = 0.6 + 0.4 * self.triangle_wave(base);
                let mut color = self.sample_palette(colors, wave);
                color = Self::scale_color(color, brightness);
                let [r, g, b] = self.ctx.apply_brightness(color);
                canvas.set_pixel(x, y, r, g, b);
            }
        }
    }

    fn render_color_fade(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        drift_speed: f32,
    ) {
        if colors.is_empty() || !drift_speed.is_finite() {
            return;
        }
        let progress = (self.elapsed * drift_speed).fract();
        let color = self.sample_palette(colors, progress);
        self.fill_canvas(canvas, color);
    }

    fn render_strobe(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        flash_ms: u32,
        fade_ms: u32,
        randomize: bool,
        randomization_factor: f32,
    ) {
        if colors.is_empty() || flash_ms == 0 || fade_ms == 0 {
            return;
        }

        let base_cycle_ms = flash_ms + fade_ms;
        let elapsed_ms = (self.elapsed * 1000.0) as u32;

        let (cycle_index, phase_ms) = if randomize && randomization_factor > 0.0 {
            self.strobe_calculate_cycle_with_randomization(
                elapsed_ms,
                base_cycle_ms,
                randomization_factor,
            )
        } else {
            let cycle_index = (elapsed_ms / base_cycle_ms) as usize;
            let phase_ms = elapsed_ms % base_cycle_ms;
            (cycle_index, phase_ms)
        };

        let palette_index = cycle_index % colors.len();

        let brightness = if phase_ms < flash_ms {
            1.0
        } else {
            let fade_progress = (phase_ms - flash_ms) as f32 / fade_ms as f32;
            (1.0 - fade_progress).clamp(0.0, 1.0)
        };

        let color = Self::scale_color(colors[palette_index], brightness);
        self.fill_canvas(canvas, color);
    }

    fn render_sparkle(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        density: f32,
        twinkle_ms: u32,
    ) {
        if colors.is_empty() || density <= 0.0 || twinkle_ms == 0 {
            return;
        }

        let width = self.width();
        let height = self.height();
        let palette_len = colors.len();
        let active_density = density.clamp(0.01, 1.0);
        let phase_base = (self.elapsed * 1000.0) / twinkle_ms as f32;

        canvas.fill(0, 0, 0);

        for y in 0..height {
            for x in 0..width {
                let seed = Self::tile_seed(y as u32, x as u32);
                if Self::pseudo_random_f32(seed) > active_density {
                    continue;
                }

                let palette_index = (seed as usize) % palette_len;
                let speed_variation =
                    0.6 + 1.2 * Self::pseudo_random_f32(seed.wrapping_mul(31_415_927));
                let phase_offset = Self::pseudo_random_f32(seed.wrapping_mul(97_531));
                let twinkle_phase = (phase_base * speed_variation + phase_offset).fract();
                let brightness = Self::sparkle_brightness(twinkle_phase);

                let mut color = colors[palette_index];
                color = Self::scale_color(color, brightness);
                let [r, g, b] = self.ctx.apply_brightness(color);
                canvas.set_pixel(x, y, r, g, b);
            }
        }
    }

    fn render_mosaic_twinkle(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        tile_size: u8,
        flow_speed: f32,
        border_size: u8,
        border_color: [u8; 3],
    ) {
        if colors.is_empty() || tile_size == 0 || !flow_speed.is_finite() || flow_speed <= 0.0 {
            return;
        }

        let width = self.width();
        let height = self.height();
        let tile = tile_size.max(1) as usize;
        let cols = (width + tile - 1) / tile;
        let rows = (height + tile - 1) / tile;
        let effective_border = if border_size == 0 {
            0
        } else {
            border_size
                .min(tile_size)
                .min(tile_size.saturating_sub(1) / 2) as usize
        };
        if border_size == 0 {
            canvas.fill(0, 0, 0);
        } else {
            let [br, bg, bb] = self.ctx.apply_brightness(border_color);
            canvas.fill(br, bg, bb);
        }

        for row in 0..rows {
            for col in 0..cols {
                let seed = Self::tile_seed(row as u32, col as u32);
                let color_idx = (seed as usize) % colors.len();
                let base_color = colors[color_idx];

                let speed_variation =
                    0.6 + 0.6 * Self::pseudo_random_f32(seed.wrapping_mul(31_415_927));
                let phase_offset = Self::pseudo_random_f32(seed.wrapping_mul(97_531));
                let phase = (self.elapsed * flow_speed * speed_variation + phase_offset).fract();
                let shimmer = 0.65 + 0.35 * (TAU * phase).sin();

                let color = Self::scale_color(base_color, shimmer.clamp(0.2, 1.0));
                let [r, g, b] = self.ctx.apply_brightness(color);

                let start_x = col * tile;
                let end_x = ((col + 1) * tile).min(width);
                let start_y = row * tile;
                let end_y = ((row + 1) * tile).min(height);

                let inner_start_x = if col == 0 {
                    start_x
                } else {
                    start_x.saturating_add(effective_border)
                };
                let inner_end_x = if col == cols - 1 {
                    end_x
                } else {
                    end_x.saturating_sub(effective_border)
                };
                let inner_start_y = if row == 0 {
                    start_y
                } else {
                    start_y.saturating_add(effective_border)
                };
                let inner_end_y = if row == rows - 1 {
                    end_y
                } else {
                    end_y.saturating_sub(effective_border)
                };

                if inner_start_x >= inner_end_x || inner_start_y >= inner_end_y {
                    // Tile too small for a full border; fall back to filling the available area.
                    for y in start_y..end_y {
                        for x in start_x..end_x {
                            canvas.set_pixel(x, y, r, g, b);
                        }
                    }
                    continue;
                }

                for y in inner_start_y..inner_end_y {
                    for x in inner_start_x..inner_end_x {
                        canvas.set_pixel(x, y, r, g, b);
                    }
                }
            }
        }
    }

    fn render_plasma(
        &self,
        canvas: &mut Box<dyn LedCanvas>,
        colors: &[[u8; 3]],
        flow_speed: f32,
        noise_scale: f32,
    ) {
        if colors.is_empty()
            || !flow_speed.is_finite()
            || flow_speed <= 0.0
            || !noise_scale.is_finite()
            || noise_scale <= 0.0
        {
            return;
        }

        let width = self.width().max(1);
        let height = self.height().max(1);
        let inv_width = 1.0 / width as f32;
        let inv_height = 1.0 / height as f32;
        let scale = noise_scale.max(0.1);
        let time = self.elapsed * flow_speed;
        let ring_scale = (scale * 0.8).max(0.2);

        for y in 0..height {
            let ny = y as f32 * inv_height;
            for x in 0..width {
                let nx = x as f32 * inv_width;

                let cx = nx - 0.5;
                let cy = ny - 0.5;
                let radius = (cx * cx + cy * cy).sqrt();
                let radius_norm = (radius * 2.0).min(1.6);
                let angle = cy.atan2(cx);
                let spin = angle + time * 0.35;

                let swirl_x = cx * spin.cos() - cy * spin.sin();
                let swirl_y = cx * spin.sin() + cy * spin.cos();

                let field_x = swirl_x * scale * 3.4 + time * 0.6;
                let field_y = swirl_y * scale * 3.4 - time * 0.45;

                let base = Self::fractal_noise(field_x, field_y, 4, 0.52, 2.1, 0x9e37_79b9);
                let polar = Self::fractal_noise(
                    radius_norm * scale * 4.8 + time * 0.35,
                    spin * 0.45 + time * 0.18,
                    3,
                    0.6,
                    2.35,
                    0x85eb_ca77,
                );

                let ring_wave =
                    ((radius_norm * ring_scale * 6.0) - time * 0.9 + spin * 0.75).sin() * 0.5 + 0.5;

                let palette_position =
                    Self::wrap01(base * 0.5 + polar * 0.35 + ring_wave * 0.15 + time * 0.05);
                let energy = (base * 0.45 + polar * 0.35 + ring_wave * 0.2).clamp(0.0, 1.0);
                let shimmer_phase = Self::wrap01(polar * 0.6 + ring_wave * 0.4 + time * 0.1);
                let shimmer = (TAU * shimmer_phase).sin() * 0.5 + 0.5;
                let brightness = 0.3 + 0.7 * (0.65 * energy + 0.35 * shimmer);

                let mut color = self.sample_palette(colors, palette_position);
                color = Self::scale_color(color, brightness);
                let [r, g, b] = self.ctx.apply_brightness(color);
                canvas.set_pixel(x, y, r, g, b);
            }
        }
    }

    fn pseudo_random_f32(seed: u32) -> f32 {
        let mut x = seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        (x as f32 / u32::MAX as f32).fract()
    }

    fn tile_seed(row: u32, col: u32) -> u32 {
        row.wrapping_mul(738_560_93) ^ col.wrapping_mul(193_496_63)
    }

    fn sparkle_brightness(phase: f32) -> f32 {
        let wave = (TAU * phase).sin() * 0.5 + 0.5;
        0.1 + 0.9 * wave.powf(2.2)
    }

    fn strobe_calculate_cycle_with_randomization(
        &self,
        elapsed_ms: u32,
        base_cycle_ms: u32,
        factor: f32,
    ) -> (usize, u32) {
        let clamped = factor.clamp(0.0, 1.0);
        let min_multiplier = 1.0 - clamped;
        let max_multiplier = 1.0 + clamped;

        // Iteratively find which cycle we're in by accumulating randomized cycle durations
        let mut accumulated_ms = 0u32;
        let mut cycle_index = 0usize;
        let max_cycles = 10000; // Safety limit to prevent infinite loops

        while accumulated_ms <= elapsed_ms && cycle_index < max_cycles {
            // Calculate randomized duration for this cycle
            let seed = Self::tile_seed(cycle_index as u32, 531_441);
            let random_value = Self::pseudo_random_f32(seed);
            let multiplier = min_multiplier + (max_multiplier - min_multiplier) * random_value;
            let cycle_duration = (base_cycle_ms as f32 * multiplier).round() as u32;
            let cycle_duration = cycle_duration.max(1); // Ensure at least 1ms

            if accumulated_ms + cycle_duration > elapsed_ms {
                // We're in this cycle
                let phase_ms = elapsed_ms - accumulated_ms;
                return (cycle_index, phase_ms);
            }

            accumulated_ms += cycle_duration;
            cycle_index += 1;
        }

        // Fallback: if we hit the limit, use approximate calculation
        let approximate_cycle =
            (elapsed_ms as f32 / (base_cycle_ms as f32 * (1.0 + clamped * 0.5))) as usize;
        let seed = Self::tile_seed(approximate_cycle as u32, 531_441);
        let random_value = Self::pseudo_random_f32(seed);
        let multiplier = min_multiplier + (max_multiplier - min_multiplier) * random_value;
        let cycle_duration = (base_cycle_ms as f32 * multiplier).round() as u32;
        let cycle_duration = cycle_duration.max(1);
        let phase_ms = elapsed_ms % cycle_duration;
        (approximate_cycle, phase_ms)
    }

    fn loop_progress(&self, cycle_s: f32) -> f32 {
        if cycle_s <= 0.0 {
            return 0.0;
        }
        (self.elapsed / cycle_s).fract()
    }

    fn triangle_wave(&self, t: f32) -> f32 {
        if t < 0.5 {
            t * 2.0
        } else {
            (1.0 - t) * 2.0
        }
    }

    fn sample_palette(&self, colors: &[[u8; 3]], position: f32) -> [u8; 3] {
        match colors.len() {
            0 => [0, 0, 0],
            1 => colors[0],
            len => {
                let pos = position.clamp(0.0, 0.9999) * len as f32;
                let idx = pos.floor() as usize;
                let frac = pos - idx as f32;
                let next = (idx + 1) % len;
                [
                    Self::lerp(colors[idx][0], colors[next][0], frac),
                    Self::lerp(colors[idx][1], colors[next][1], frac),
                    Self::lerp(colors[idx][2], colors[next][2], frac),
                ]
            }
        }
    }

    fn lerp(a: u8, b: u8, t: f32) -> u8 {
        ((a as f32 * (1.0 - t)) + (b as f32 * t))
            .round()
            .clamp(0.0, 255.0) as u8
    }

    fn scale_color(color: [u8; 3], brightness: f32) -> [u8; 3] {
        let b = brightness.clamp(0.0, 1.0);
        [
            (color[0] as f32 * b) as u8,
            (color[1] as f32 * b) as u8,
            (color[2] as f32 * b) as u8,
        ]
    }

    fn wrap01(value: f32) -> f32 {
        let mut v = value % 1.0;
        if v < 0.0 {
            v += 1.0;
        }
        v
    }

    fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn smoothstep(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    fn hash_coords(x: i32, y: i32, salt: u32) -> f32 {
        let mut n = x as u32;
        n = n
            .wrapping_mul(374_761_393)
            .wrapping_add((y as u32) ^ 668_265_263);
        n ^= n >> 13;
        n ^= n << 17;
        n ^= n >> 5;
        n ^= salt;
        (n as f32 / u32::MAX as f32).clamp(0.0, 1.0)
    }

    fn value_noise(x: f32, y: f32, salt: u32) -> f32 {
        let x0 = x.floor();
        let y0 = y.floor();
        let x1 = x0 + 1.0;
        let y1 = y0 + 1.0;
        let sx = Self::smoothstep(x - x0);
        let sy = Self::smoothstep(y - y0);

        let n00 = Self::hash_coords(x0 as i32, y0 as i32, salt);
        let n10 = Self::hash_coords(x1 as i32, y0 as i32, salt);
        let n01 = Self::hash_coords(x0 as i32, y1 as i32, salt);
        let n11 = Self::hash_coords(x1 as i32, y1 as i32, salt);

        let ix0 = Self::lerp_f32(n00, n10, sx);
        let ix1 = Self::lerp_f32(n01, n11, sx);
        Self::lerp_f32(ix0, ix1, sy)
    }

    fn fractal_noise(
        x: f32,
        y: f32,
        octaves: u8,
        persistence: f32,
        lacunarity: f32,
        salt: u32,
    ) -> f32 {
        if octaves == 0 {
            return 0.0;
        }

        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_amplitude = 0.0;
        let mut total = 0.0;

        for octave in 0..octaves {
            let sample = Self::value_noise(
                x * frequency,
                y * frequency,
                salt.wrapping_add(octave as u32),
            );
            total += sample * amplitude;
            max_amplitude += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        if max_amplitude <= f32::EPSILON {
            0.0
        } else {
            (total / max_amplitude).clamp(0.0, 1.0)
        }
    }
}
