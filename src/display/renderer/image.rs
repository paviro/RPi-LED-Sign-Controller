use log::{debug, error, warn};
use std::path::Path;

use crate::display::driver::LedCanvas;
use crate::display::renderer::{RenderContext, Renderer};
use crate::models::content::ContentDetails;
use crate::models::image::{ImageAnimation, ImageContent, ImageTransform};
use crate::models::playlist::PlayListItem;
use crate::storage::manager::{paths, DEFAULT_DIR};

const MIN_SCALE: f32 = 0.01;

struct DecodedImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl DecodedImage {
    fn sample(&self, x: u32, y: u32) -> [u8; 3] {
        let idx = ((y * self.width + x) * 3) as usize;
        [self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2]]
    }
}

#[derive(Clone, Copy, Debug)]
struct PreciseTransform {
    x: f32,
    y: f32,
    scale: f32,
}

impl From<&ImageTransform> for PreciseTransform {
    fn from(transform: &ImageTransform) -> Self {
        Self {
            x: transform.x as f32,
            y: transform.y as f32,
            scale: transform.scale,
        }
    }
}

pub struct ImageRenderer {
    ctx: RenderContext,
    content: ImageContent,
    decoded: Option<DecodedImage>,
    duration_seconds: Option<u64>,
    elapsed_seconds: f32,
    animation_elapsed_ms: f32,
    completed_iterations: u32,
    max_iterations: Option<u32>,
    is_complete: bool,
}

impl Renderer for ImageRenderer {
    fn new(content: &PlayListItem, ctx: RenderContext) -> Self
    where
        Self: Sized,
    {
        let image_content = match &content.content.data {
            ContentDetails::Image(image_content) => image_content.clone(),
            _ => unreachable!("ImageRenderer can only be created with image content"),
        };

        let decoded = load_image(&image_content.image_id);
        if decoded.is_none() {
            warn!(
                "Failed to load image {} for playlist item {}",
                image_content.image_id, content.id
            );
        } else {
            debug!(
                "Loaded image {} ({}x{})",
                image_content.image_id, image_content.natural_width, image_content.natural_height
            );
        }

        Self {
            ctx,
            content: image_content,
            decoded,
            duration_seconds: content.duration,
            elapsed_seconds: 0.0,
            animation_elapsed_ms: 0.0,
            completed_iterations: 0,
            max_iterations: repeat_count_to_iterations(content.repeat_count),
            is_complete: false,
        }
    }

    fn update(&mut self, dt: f32) {
        if self.decoded.is_none() {
            self.is_complete = true;
            return;
        }

        if self.is_complete {
            return;
        }

        if let Some(duration) = self.duration_seconds {
            self.elapsed_seconds += dt;
            if self.elapsed_seconds >= duration as f32 {
                self.is_complete = true;
            }
        }

        if let Some(animation) = &self.content.animation {
            if animation.keyframes.len() >= 2 {
                self.animation_elapsed_ms += dt * 1000.0;
                let cycle_length = animation_length_ms(animation).max(1) as f32;
                while self.animation_elapsed_ms >= cycle_length {
                    self.completed_iterations = self.completed_iterations.saturating_add(1);

                    let reached_repeat_limit = self
                        .max_iterations
                        .map(|max_iters| max_iters != 0 && self.completed_iterations >= max_iters)
                        .unwrap_or(false);

                    if reached_repeat_limit || self.is_complete {
                        self.animation_elapsed_ms = cycle_length;
                        self.is_complete = true;
                        break;
                    }

                    self.animation_elapsed_ms -= cycle_length;
                }
            }
        }
    }

    fn render(&self, canvas: &mut Box<dyn LedCanvas>) {
        let decoded = match &self.decoded {
            Some(image) => image,
            None => return,
        };

        let transform = self.current_transform();
        let scale = transform.scale.max(MIN_SCALE);
        let scaled_width = decoded.width as f32 * scale;
        let scaled_height = decoded.height as f32 * scale;

        let start_x = transform.x.floor() as i32;
        let mut end_x = (transform.x + scaled_width).ceil() as i32;
        if end_x <= start_x {
            end_x = start_x + 1;
        }

        let start_y = transform.y.floor() as i32;
        let mut end_y = (transform.y + scaled_height).ceil() as i32;
        if end_y <= start_y {
            end_y = start_y + 1;
        }

        for panel_y in start_y..end_y {
            if panel_y < 0 || panel_y >= self.ctx.display_height {
                continue;
            }

            let src_y = (((panel_y as f32) - transform.y) / scale)
                .floor()
                .clamp(0.0, decoded.height as f32 - 1.0) as u32;

            for panel_x in start_x..end_x {
                if panel_x < 0 || panel_x >= self.ctx.display_width {
                    continue;
                }

                let src_x = (((panel_x as f32) - transform.x) / scale)
                    .floor()
                    .clamp(0.0, decoded.width as f32 - 1.0) as u32;

                let color = decoded.sample(src_x, src_y);
                let [r, g, b] = self.ctx.apply_brightness(color);
                canvas.set_pixel(panel_x as usize, panel_y as usize, r, g, b);
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn reset(&mut self) {
        self.elapsed_seconds = 0.0;
        self.animation_elapsed_ms = 0.0;
        self.completed_iterations = 0;
        self.is_complete = false;
    }

    fn update_context(&mut self, ctx: RenderContext) {
        self.ctx = ctx;
    }

    fn update_content(&mut self, content: &PlayListItem) {
        if let ContentDetails::Image(image_content) = &content.content.data {
            if self.content.image_id != image_content.image_id {
                self.decoded = load_image(&image_content.image_id);
            }
            self.content = image_content.clone();
            self.duration_seconds = content.duration;
            self.max_iterations = repeat_count_to_iterations(content.repeat_count);
            self.reset();
        }
    }
}

impl ImageRenderer {
    fn current_transform(&self) -> PreciseTransform {
        if let Some(animation) = &self.content.animation {
            if animation.keyframes.len() >= 2 {
                if let Some(transform) = interpolate_transform(animation, self.animation_elapsed_ms)
                {
                    return transform;
                }
            }
        }
        PreciseTransform::from(&self.content.transform)
    }
}

fn repeat_count_to_iterations(repeat_count: Option<u32>) -> Option<u32> {
    match repeat_count {
        Some(0) | None => None,
        Some(value) => Some(value),
    }
}

fn interpolate_transform(animation: &ImageAnimation, elapsed_ms: f32) -> Option<PreciseTransform> {
    if animation.keyframes.len() < 2 {
        return None;
    }

    let mut previous = &animation.keyframes[0];
    for next in animation.keyframes.iter().skip(1) {
        if elapsed_ms <= next.timestamp_ms as f32 {
            let segment_duration =
                (next.timestamp_ms.saturating_sub(previous.timestamp_ms)).max(1) as f32;
            let progress =
                ((elapsed_ms - previous.timestamp_ms as f32) / segment_duration).clamp(0.0, 1.0);

            return Some(PreciseTransform {
                x: lerp(previous.x as f32, next.x as f32, progress),
                y: lerp(previous.y as f32, next.y as f32, progress),
                scale: lerp(previous.scale, next.scale, progress).max(MIN_SCALE),
            });
        }
        previous = next;
    }

    animation.keyframes.last().map(|last| PreciseTransform {
        x: last.x as f32,
        y: last.y as f32,
        scale: last.scale.max(MIN_SCALE),
    })
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn animation_length_ms(animation: &ImageAnimation) -> u32 {
    animation
        .keyframes
        .last()
        .map(|keyframe| keyframe.timestamp_ms)
        .unwrap_or(0)
}

fn load_image(image_id: &str) -> Option<DecodedImage> {
    let base_dir = std::env::var("LED_STORAGE_DIR").unwrap_or_else(|_| DEFAULT_DIR.to_string());
    let path = Path::new(&base_dir)
        .join(paths::IMAGES_DIR)
        .join(format!("{}.png", image_id));

    match image::open(&path) {
        Ok(dynamic) => {
            let rgb = dynamic.to_rgb8();
            let width = rgb.width();
            let height = rgb.height();
            Some(DecodedImage {
                width,
                height,
                pixels: rgb.into_raw(),
            })
        }
        Err(err) => {
            error!("Failed to open image {}: {}", path.display(), err);
            None
        }
    }
}
