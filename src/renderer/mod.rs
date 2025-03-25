mod text;
mod border;
mod context;

pub use text::TextRenderer;
pub use border::BorderRenderer;
pub use context::RenderContext;

use crate::models::{DisplayContent, ContentType, ContentDetails};
use crate::led_driver::LedCanvas;

/// Core Renderer trait that all content-specific renderers must implement
pub trait Renderer: Send + Sync {
    /// Initialize a new renderer instance with content and context
    fn new(content: &DisplayContent, ctx: RenderContext) -> Self where Self: Sized;
    
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
    fn update_content(&mut self, content: &DisplayContent);
}

/// Factory function to create the appropriate content renderer based on content type
pub fn create_renderer(content: &DisplayContent, ctx: RenderContext) -> Box<dyn Renderer> {
    match content.content.content_type {
        ContentType::Text => {
            match &content.content.data {
                ContentDetails::Text(_) => {
                    Box::new(TextRenderer::new(content, ctx))
                },
                #[allow(unreachable_patterns)]
                _ => panic!("Content type mismatch: expected Text content details")
            }
        },
        // Future content types will be added here
    }
}

/// Create a border renderer for the given content
pub fn create_border_renderer(content: &DisplayContent, ctx: RenderContext) -> Box<dyn Renderer> {
    Box::new(BorderRenderer::new(content, ctx))
} 