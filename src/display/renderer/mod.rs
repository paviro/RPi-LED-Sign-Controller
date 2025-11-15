mod border;
mod clock;
mod context;
mod image;
mod text;

pub use border::BorderRenderer;
pub use clock::ClockRenderer;
pub use context::RenderContext;
pub use image::ImageRenderer;
pub use text::TextRenderer;

use crate::display::driver::LedCanvas;
use crate::models::content::{ContentDetails, ContentType};
use crate::models::playlist::PlayListItem;

/// Core Renderer trait that all content-specific renderers must implement
pub trait Renderer: Send + Sync {
    /// Initialize a new renderer instance with content and context
    fn new(content: &PlayListItem, ctx: RenderContext) -> Self
    where
        Self: Sized;

    /// Update renderer state based on elapsed time
    fn update(&mut self, dt: f32);

    /// Render content to the provided canvas
    fn render(&self, canvas: &mut Box<dyn LedCanvas>);

    /// Check if content has completed its display cycle
    /// Used for determining transitions between playlist items
    fn is_complete(&self) -> bool;

    /// Reset the renderer to its initial state
    /// Called when content transitions occur
    fn reset(&mut self);

    /// Update the renderer's context without resetting animation state
    fn update_context(&mut self, ctx: RenderContext);

    /// Update the renderer's content without fully resetting animation state
    fn update_content(&mut self, content: &PlayListItem);
}

/// Factory function to create the appropriate content renderer based on content type
pub fn create_renderer(content: &PlayListItem, ctx: RenderContext) -> Box<dyn Renderer> {
    match content.content.content_type {
        ContentType::Text => match &content.content.data {
            ContentDetails::Text(_) => Box::new(TextRenderer::new(content, ctx)),
            #[allow(unreachable_patterns)]
            _ => panic!("Content type mismatch: expected Text content details"),
        },
        ContentType::Image => match &content.content.data {
            ContentDetails::Image(_) => Box::new(ImageRenderer::new(content, ctx)),
            #[allow(unreachable_patterns)]
            _ => panic!("Content type mismatch: expected Image content details"),
        },
        ContentType::Clock => match &content.content.data {
            ContentDetails::Clock(_) => Box::new(ClockRenderer::new(content, ctx)),
            #[allow(unreachable_patterns)]
            _ => panic!("Content type mismatch: expected Clock content details"),
        },
    }
}

/// Create a border renderer for the given content
pub fn create_border_renderer(content: &PlayListItem, ctx: RenderContext) -> Box<dyn Renderer> {
    Box::new(BorderRenderer::new(content, ctx))
}
