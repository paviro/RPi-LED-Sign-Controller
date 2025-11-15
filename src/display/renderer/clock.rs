use crate::display::driver::LedCanvas;
use crate::display::graphics::embedded_graphics_support::EmbeddedGraphicsCanvas;
use crate::display::renderer::{RenderContext, Renderer};
use crate::models::clock::{ClockContent, ClockFormat};
use crate::models::content::ContentDetails;
use crate::models::playlist::PlayListItem;
use chrono::Local;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::iso_8859_1::FONT_10X20 as FONT_10X20_LATIN1;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use log::warn;
use std::time::Instant;

pub struct ClockRenderer {
    content: ClockContent,
    ctx: RenderContext,
    duration: Option<u64>,
    start_time: Instant,
}

impl Renderer for ClockRenderer {
    fn new(content: &PlayListItem, ctx: RenderContext) -> Self {
        let clock_content = match &content.content.data {
            ContentDetails::Clock(clock) => clock.clone(),
            #[allow(unreachable_patterns)]
            _ => panic!("Expected clock content"),
        };

        Self {
            content: clock_content,
            ctx: ctx.clone(),
            duration: content.duration,
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, _dt: f32) {
        // No animation state required; rendering uses current system time
    }

    fn render(&self, canvas: &mut Box<dyn LedCanvas>) {
        let mut eg_canvas = EmbeddedGraphicsCanvas::new(canvas);
        let time_str = self.format_time_string();

        let font = &FONT_10X20_LATIN1;
        let char_width = font.character_size.width as i32;
        let font_height = font.character_size.height as i32;
        let text_width = (time_str.chars().count() as i32) * char_width;
        let x = (self.ctx.display_width - text_width) / 2;
        let y = self.ctx.calculate_centered_text_position(font_height);
        let [r, g, b] = self.ctx.apply_brightness(self.content.color);
        let text_style = MonoTextStyle::new(font, Rgb888::new(r, g, b));

        let _ = Text::new(&time_str, Point::new(x, y), text_style).draw(&mut eg_canvas);
    }

    fn is_complete(&self) -> bool {
        if let Some(duration) = self.duration {
            return Instant::now().duration_since(self.start_time).as_secs() >= duration;
        }
        false
    }

    fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    fn update_context(&mut self, ctx: RenderContext) {
        self.ctx = ctx;
    }

    fn update_content(&mut self, content: &PlayListItem) {
        if let ContentDetails::Clock(clock) = &content.content.data {
            self.content = clock.clone();
            self.duration = content.duration;
            self.start_time = Instant::now();
        } else {
            warn!("ClockRenderer received non-clock content during update");
        }
    }
}

impl ClockRenderer {
    fn format_time_string(&self) -> String {
        let now = Local::now();
        let show_seconds = self.content.show_seconds;

        let raw = match self.content.format {
            ClockFormat::TwentyFourHour => {
                if show_seconds {
                    now.format("%H:%M:%S").to_string()
                } else {
                    now.format("%H:%M").to_string()
                }
            }
            ClockFormat::TwelveHour => {
                let formatted = if show_seconds {
                    now.format("%I:%M:%S %p").to_string()
                } else {
                    now.format("%I:%M %p").to_string()
                };
                formatted
            }
        };

        if matches!(self.content.format, ClockFormat::TwelveHour) && raw.starts_with('0') {
            raw.trim_start_matches('0').to_string()
        } else {
            raw
        }
    }
}
