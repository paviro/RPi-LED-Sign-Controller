use crate::config::DisplayConfig;
use crate::display::driver::{LedCanvas, LedDriver};
use crate::display::renderer::{create_border_renderer, create_renderer, RenderContext, Renderer};
use crate::models::border_effects::BorderEffect;
use crate::models::content::{ContentData, ContentDetails, ContentType};
use crate::models::playlist::{PlayListItem, Playlist};
use crate::models::text::TextContent;
use log::{debug, info};
use once_cell::sync::Lazy;
use std::time::Instant;
use uuid::Uuid;

// Structure to manage LED matrix state
pub struct DisplayManager {
    pub playlist: Playlist,
    driver: Box<dyn LedDriver>,
    pub canvas: Option<Box<dyn LedCanvas>>,
    pub display_width: i32,
    pub display_height: i32,
    pub last_transition: Instant,
    pub current_repeat: u32,
    config: DisplayConfig,
    preview_mode: bool,
    preview_content: Option<PlayListItem>,
    last_preview_ping: Instant,
    active_renderer: Option<Box<dyn Renderer>>,
    border_renderer: Option<Box<dyn Renderer>>,
    preview_renderer: Option<Box<dyn Renderer>>,
    preview_border_renderer: Option<Box<dyn Renderer>>,
    render_context: RenderContext,
    preview_session_id: Option<String>,
}

impl DisplayManager {
    pub fn with_config_and_driver(config: &DisplayConfig, driver: Box<dyn LedDriver>) -> Self {
        // Get display dimensions
        let display_width = config.display_width();
        let display_height = config.display_height();

        info!(
            "Initializing display: {}x{} (rows={}, cols={}, chain={}, parallel={})",
            display_width,
            display_height,
            config.rows,
            config.cols,
            config.chain_length,
            config.parallel
        );

        // Get the canvas from the driver
        let mut driver_box = driver;
        let canvas = driver_box.take_canvas();

        // Get default playlist
        let default_playlist = Playlist::default();

        // Create render context
        let render_context =
            RenderContext::new(display_width, display_height, config.user_brightness);

        let mut display_manager = Self {
            playlist: default_playlist,
            driver: driver_box,
            canvas,
            display_width,
            display_height,
            last_transition: Instant::now(),
            current_repeat: 0,
            config: config.clone(),
            // Initialize preview mode fields
            preview_mode: false,
            preview_content: None,
            last_preview_ping: Instant::now(),
            // Initialize renderer fields
            active_renderer: None,
            border_renderer: None,
            preview_renderer: None,
            preview_border_renderer: None,
            render_context,
            preview_session_id: None,
        };

        // Initialize renderer if we have content
        display_manager.setup_active_renderer();

        display_manager
    }

    pub fn with_playlist_config_and_driver(
        playlist: Playlist,
        config: &DisplayConfig,
        driver: Box<dyn LedDriver>,
    ) -> Self {
        // Log the playlist content to diagnose the issue
        info!("Got the following {} items:", playlist.items.len());
        for (i, item) in playlist.items.iter().enumerate() {
            let content_desc = match &item.content.data {
                ContentDetails::Text(text_content) => {
                    let preview = if text_content.text.len() > 30 {
                        format!("{}...", &text_content.text[..27])
                    } else {
                        text_content.text.clone()
                    };
                    format!("Text: \"{}\"", preview)
                }
                ContentDetails::Image(image_content) => format!(
                    "Image: {} ({}x{})",
                    image_content.image_id,
                    image_content.natural_width,
                    image_content.natural_height
                ),
            };
            info!("  Item {}: {}", i + 1, content_desc);
        }

        // Create a new display manager with the given config
        let mut display_manager = Self::with_config_and_driver(config, driver);

        // Update the playlist
        display_manager.playlist = playlist;

        // IMPORTANT: Ensure we always start with the first item
        display_manager.playlist.active_index = 0;

        // Initialize renderer
        display_manager.setup_active_renderer();

        display_manager
    }

    pub fn get_current_content(&self) -> &PlayListItem {
        // If we're in preview mode, show the preview content
        if self.preview_mode && self.preview_content.is_some() {
            return self.preview_content.as_ref().unwrap();
        }

        if self.playlist.items.is_empty() {
            // Store the default message item
            static DEFAULT_ITEM: Lazy<PlayListItem> = Lazy::new(|| {
                // Get the local IP for a more helpful message
                let ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());

                PlayListItem {
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

        // Check if the current content is complete based on renderer state
        let should_transition = self
            .active_renderer
            .as_ref()
            .map_or(false, |renderer| renderer.is_complete());

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

        // Reset the static counter when switching items
        use std::sync::atomic::{AtomicU32, Ordering};
        static LAST_LOGGED_CYCLE: AtomicU32 = AtomicU32::new(0);
        LAST_LOGGED_CYCLE.store(0, Ordering::Relaxed);

        // After updating the playlist index, set up a new renderer
        self.setup_active_renderer();

        // Very important: Reset the progress tracking for the new active item
        if let Some(renderer) = &mut self.active_renderer {
            renderer.reset();
        }
    }

    pub fn update_display(&mut self) {
        let mut canvas = self.canvas.take().expect("Canvas missing");
        canvas.fill(0, 0, 0); // Clear the canvas

        // Use the appropriate content renderer
        let content_renderer = if self.preview_mode && self.preview_renderer.is_some() {
            self.preview_renderer.as_ref()
        } else {
            self.active_renderer.as_ref()
        };

        // Render content first
        if let Some(renderer) = content_renderer {
            renderer.render(&mut canvas);
        }

        // Use the appropriate border renderer
        let border_renderer = if self.preview_mode && self.preview_border_renderer.is_some() {
            self.preview_border_renderer.as_ref()
        } else {
            self.border_renderer.as_ref()
        };

        // Render border on top
        if let Some(renderer) = border_renderer {
            renderer.render(&mut canvas);
        }

        // Update the canvas using the driver
        let updated_canvas = self.driver.update_canvas(canvas);
        self.canvas = Some(updated_canvas);
    }

    // Set up the renderer for the active content
    pub fn setup_active_renderer(&mut self) {
        if self.playlist.items.is_empty() {
            self.active_renderer = None;
            self.border_renderer = None;
            return;
        }

        let current = self.get_current_content().clone();

        // Drop existing renderers first to avoid borrow conflicts
        self.active_renderer = None;
        self.border_renderer = None;

        // Then create new renderers
        self.active_renderer = Some(create_renderer(&current, self.render_context.clone()));

        // Create border renderer if border effect is specified
        if current.border_effect.is_some() {
            self.border_renderer = Some(create_border_renderer(
                &current,
                self.render_context.clone(),
            ));
        }
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
            self.update_display();
        }

        // Then shut down the driver
        self.driver.shutdown();
    }

    // Set brightness now updates the render context without resetting animations
    pub fn set_brightness(&mut self, brightness: u8) {
        let brightness = brightness.clamp(0, 100);

        // Only log at debug level for continuous updates
        // This won't show up unless RUST_LOG=debug is set
        debug!("Updating display brightness: {}", brightness);

        // Update the brightness in the config
        self.config.user_brightness = brightness;

        // Update the render context brightness
        self.render_context =
            RenderContext::new(self.display_width, self.display_height, brightness);

        // Update context in all active renderers without resetting animation state
        if let Some(renderer) = &mut self.active_renderer {
            renderer.update_context(self.render_context.clone());
        }

        if let Some(renderer) = &mut self.border_renderer {
            renderer.update_context(self.render_context.clone());
        }

        // Update preview renderers if in preview mode
        if self.preview_mode {
            if let Some(renderer) = &mut self.preview_renderer {
                renderer.update_context(self.render_context.clone());
            }

            if let Some(renderer) = &mut self.preview_border_renderer {
                renderer.update_context(self.render_context.clone());
            }
        }
    }

    // Private helper method to handle common preview content update logic
    fn update_preview_renderers(&mut self, content: &PlayListItem) {
        // Determine if the content type changed between the previous and new content
        let previous_type = self
            .preview_content
            .as_ref()
            .map(|c| c.content.content_type.clone());
        let new_type = content.content.content_type.clone();
        let content_type_changed = previous_type.map_or(true, |t| t != new_type);

        // If the content type has changed, replace the renderer to avoid panics in update_content
        // Otherwise, update the existing renderer in place to preserve animation state where possible
        match (&mut self.preview_renderer, content_type_changed) {
            (Some(renderer), false) => {
                renderer.update_content(content);
            }
            _ => {
                // Create new renderer if none exists or if the type changed
                self.preview_renderer = Some(create_renderer(content, self.render_context.clone()));
            }
        };

        // Update border renderer or create new one if needed
        if content.border_effect.is_some() {
            if let Some(renderer) = &mut self.preview_border_renderer {
                renderer.update_content(content);
            } else {
                // Create new border renderer if none exists
                self.preview_border_renderer =
                    Some(create_border_renderer(content, self.render_context.clone()));
            }
        } else {
            // Remove border renderer if no longer needed
            self.preview_border_renderer = None;
        }

        // Update the content
        self.preview_content = Some(content.clone());

        // Update the ping time
        self.last_preview_ping = Instant::now();
    }

    // Handle content preview with scroll position preservation where possible
    pub fn enter_preview_mode(&mut self, content: PlayListItem, session_id: String) {
        let already_in_preview = self.preview_mode;
        self.preview_mode = true;
        self.preview_session_id = Some(session_id.clone());

        if !already_in_preview {
            // First-time preview mode setup
            info!("Entering preview mode with session_id: {}", session_id);
        }

        // Use the common helper method
        self.update_preview_renderers(&content);
    }

    // Method to update preview content without changing the session ID
    pub fn update_preview_content(&mut self, content: PlayListItem) {
        if !self.preview_mode {
            return;
        }

        // Use the common helper method
        self.update_preview_renderers(&content);
    }

    // Update renderer state
    pub fn update_renderer(&mut self, dt: f32) {
        // Update renderers with the elapsed time
        if let Some(renderer) = &mut self.active_renderer {
            renderer.update(dt);
        }

        // Update the border renderer
        if let Some(renderer) = &mut self.border_renderer {
            renderer.update(dt);
        }

        // Update preview renderers if active
        if self.preview_mode {
            if let Some(renderer) = &mut self.preview_renderer {
                renderer.update(dt);
            }

            if let Some(renderer) = &mut self.preview_border_renderer {
                renderer.update(dt);
            }
        }
    }

    // Check if preview mode has timed out from inactivity
    pub fn check_preview_timeout(&mut self, timeout_seconds: u64) -> Option<String> {
        if self.preview_mode {
            let elapsed = self.last_preview_ping.elapsed().as_secs();
            if elapsed > timeout_seconds {
                info!(
                    "Preview mode timed out after {} seconds of inactivity",
                    elapsed
                );
                // Store session ID before exiting preview mode
                let session_id = self.preview_session_id.clone();
                self.exit_preview_mode();
                return session_id;
            }
        }
        None
    }

    // Check if preview mode is currently active
    pub fn is_in_preview_mode(&self) -> bool {
        self.preview_mode
    }

    // Update the ping time and return whether the operation was successful
    pub fn update_preview_ping(&mut self) -> bool {
        if self.preview_mode {
            self.last_preview_ping = Instant::now();
            true
        } else {
            false
        }
    }

    // Add this public method that handlers.rs calls
    pub fn reset_display_state(&mut self) {
        // Reset the display state to start fresh with current item
        self.last_transition = Instant::now();
        self.current_repeat = 0;

        // Reset the active renderers
        if let Some(renderer) = &mut self.active_renderer {
            renderer.reset();
        }
        if let Some(renderer) = &mut self.border_renderer {
            renderer.reset();
        }

        // Setup renderers (this might create new renderers)
        self.setup_active_renderer();
    }

    // Add a method to check if a session owns the preview
    pub fn is_preview_session_owner(&self, session_id: &str) -> bool {
        if !self.preview_mode {
            return false;
        }

        self.preview_session_id
            .as_ref()
            .map_or(false, |id| id == session_id)
    }

    pub fn exit_preview_mode(&mut self) {
        if self.preview_mode {
            info!(
                "Exiting preview mode for session_id: {}",
                self.preview_session_id.clone().unwrap_or_default()
            );
            self.preview_mode = false;
            self.preview_content = None;
            self.preview_renderer = None;
            self.preview_border_renderer = None;
            self.preview_session_id = None;
        }
    }
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
        }
        Err(_) => None,
    }
}
