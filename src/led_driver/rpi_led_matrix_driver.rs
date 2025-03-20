use std::fmt::Debug;
use std::any::Any;
use log::{debug, error, warn};
use rpi_led_matrix::{LedMatrix, LedMatrixOptions, LedCanvas as RpiCanvas, 
                      LedColor, LedRuntimeOptions};

use crate::config::DisplayConfig;
use super::{LedCanvas, LedDriver};
use super::options::MatrixOptions;

// Canvas implementation for rpi-led-matrix
pub struct RpiLedMatrixCanvas {
    canvas: RpiCanvas,
    width: i32,
    height: i32,
}

// Manual Debug impl
impl Debug for RpiLedMatrixCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpiLedMatrixCanvas")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

// Explicitly implement Send for thread safety
unsafe impl Send for RpiLedMatrixCanvas {}

impl LedCanvas for RpiLedMatrixCanvas {
    fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        // The rpi-led-matrix set method takes a color struct and i32 coordinates
        let color = LedColor { red: r, green: g, blue: b };
        self.canvas.set(x as i32, y as i32, &color);
    }

    fn fill(&mut self, r: u8, g: u8, b: u8) {
        // Fill with a color
        let color = LedColor { red: r, green: g, blue: b };
        self.canvas.fill(&color);
    }

    fn size(&self) -> (i32, i32) {
        // Use stored values instead of method call
        (self.width, self.height)
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any where Self: 'static {
        self
    }
}

// Driver implementation for rpi-led-matrix
pub struct RpiLedMatrixDriver {
    matrix: LedMatrix,
    width: i32,
    height: i32,
}

// Manual Debug impl
impl Debug for RpiLedMatrixDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpiLedMatrixDriver")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

// Explicitly implement Send for thread safety
unsafe impl Send for RpiLedMatrixDriver {}

impl LedDriver for RpiLedMatrixDriver {
    fn initialize(config: &DisplayConfig) -> Result<Self, String> {
        // Get common options
        let options = MatrixOptions::from_config(config);
        
        // Create binding driver specific options
        let (matrix_options, rt_options) = Self::create_matrix_options(&options)?;
        
        debug!("Initializing rpi-led-matrix with options: {:?}", options);
        
        // Pass the runtime options to the initialization
        match LedMatrix::new(Some(matrix_options), Some(rt_options)) {
            Ok(matrix) => {
                // Use options values for dimensions
                let width = (options.cols * options.chain_length) as i32;
                let height = (options.rows * options.parallel) as i32;
                
                Ok(Self {
                    matrix,
                    width,
                    height,
                })
            },
            Err(e) => Err(format!("Failed to initialize rpi-led-matrix: {}", e)),
        }
    }

    fn take_canvas(&mut self) -> Option<Box<dyn LedCanvas>> {
        let canvas = self.matrix.offscreen_canvas();
        // Use stored dimension values
        Some(Box::new(RpiLedMatrixCanvas {
            canvas,
            width: self.width,
            height: self.height,
        }))
    }

    fn update_canvas(&mut self, mut canvas: Box<dyn LedCanvas>) -> Box<dyn LedCanvas> {
        // Get the specific canvas type
        let matrix_canvas: &mut RpiLedMatrixCanvas = canvas
            .as_any_mut()
            .downcast_mut::<RpiLedMatrixCanvas>()
            .expect("Canvas was not an RpiLedMatrixCanvas");
        
        // Save dimensions
        let width = matrix_canvas.width;
        let height = matrix_canvas.height;
        
        // Take ownership of the canvas in a safer way
        // We'll need to create a new RpiCanvas as a temporary placeholder
        let old_canvas = std::mem::replace(
            &mut matrix_canvas.canvas, 
            // Creating a new temporary canvas - in practice this won't be used
            self.matrix.offscreen_canvas()
        );
        
        // Swap canvases and get the new one
        let new_canvas = self.matrix.swap(old_canvas);
        
        // Return new wrapped canvas with same dimensions
        Box::new(RpiLedMatrixCanvas {
            canvas: new_canvas,
            width,
            height,
        })
    }

    fn shutdown(&mut self) {
        // For the binding driver, clear the display
        let mut canvas = self.matrix.offscreen_canvas();
        let color = LedColor { red: 0, green: 0, blue: 0 }; // Black
        canvas.fill(&color);
        
        // Make sure to capture the return value even if we don't use it
        let _new_canvas = self.matrix.swap(canvas);
        
        // Sleep briefly to ensure the display is updated
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

impl RpiLedMatrixDriver {
    // Create driver-specific options from common options
    fn create_matrix_options(options: &MatrixOptions) -> Result<(LedMatrixOptions, LedRuntimeOptions), String> {
        let mut matrix_options = LedMatrixOptions::new();
        let mut rt_options = LedRuntimeOptions::new();
        let mut unsupported_options = Vec::new();
        
        // Apply basic panel options
        matrix_options.set_rows(options.rows as u32);
        matrix_options.set_cols(options.cols as u32);
        matrix_options.set_chain_length(options.chain_length as u32);
        
        // Validate parallel chains - binding only supports 1-3 chains
        if options.parallel > 3 {
            return Err(format!(
                "C++ binding driver only supports 1-3 parallel chains, but {} was specified", 
                options.parallel
            ));
        }
        matrix_options.set_parallel(options.parallel as u32);
        
        // Set brightness (1-100)
        if let Err(e) = matrix_options.set_brightness(options.brightness) {
            return Err(format!("Failed to set brightness: {}", e));
        }
        
        // Apply hardware mapping
        matrix_options.set_hardware_mapping(&options.hardware_mapping);
        
        // Apply GPIO slowdown if specified
        if let Some(slowdown) = options.gpio_slowdown {
            rt_options.set_gpio_slowdown(slowdown);
        }
        
        // Apply PWM bits (with error handling)
        if let Err(e) = matrix_options.set_pwm_bits(options.pwm_bits) {
            error!("Failed to set PWM bits: {}", e);
            unsupported_options.push(format!("pwm_bits={}", options.pwm_bits));
        }
        
        // Apply PWM LSB nanoseconds
        matrix_options.set_pwm_lsb_nanoseconds(options.pwm_lsb_nanoseconds);
        
        // Apply scan mode (interlaced)
        matrix_options.set_scan_mode(if options.interlaced { 1 } else { 0 });
        
        // Apply dither bits
        matrix_options.set_pwm_dither_bits(options.dither_bits as u32);
        
        // Apply panel type if specified
        if let Some(panel) = &options.panel_type {
            // The C++ binding accepts panel types as strings directly
            matrix_options.set_panel_type(panel);
        }
        
        // Apply pixel mapper if specified
        if let Some(mapper) = &options.pixel_mapper {
            // The C++ binding accepts mappers as a semicolon-separated string
            matrix_options.set_pixel_mapper_config(mapper);
        }
        
        // Apply multiplexing if specified
        if let Some(multiplex_str) = &options.multiplexing {
            let multiplex_val = Self::map_multiplexing(multiplex_str);
            matrix_options.set_multiplexing(multiplex_val);
        }
        
        // Apply LED sequence
        matrix_options.set_led_rgb_sequence(&options.led_sequence);
        
        // Apply row address type
        let row_addr_val = Self::map_row_setter(&options.row_setter);
        matrix_options.set_row_addr_type(row_addr_val);
        
        // Apply hardware pulsing (default is true, CLI flag disables it)
        matrix_options.set_hardware_pulsing(options.hardware_pulsing);
        
        // Apply refresh rate stats display
        matrix_options.set_refresh_rate(options.show_refresh);
        
        // Apply inverse colors
        matrix_options.set_inverse_colors(options.inverse_colors);
        
        // Apply refresh rate limiting
        if options.limit_refresh_rate > 0 {
            matrix_options.set_limit_refresh(options.limit_refresh_rate);
        }
        
        // Runtime options: set reasonable defaults
        rt_options.set_drop_privileges(true); // Drop privileges after initialization
        
        // Check if we encountered any unsupported options
        if !unsupported_options.is_empty() {
            return Err(format!(
                "The following options are not supported by the binding driver: {}",
                unsupported_options.join(", ")
            ));
        }
        
        Ok((matrix_options, rt_options))
    }
    
    // Helper to map multiplexing string to numeric value
    fn map_multiplexing(multiplex_str: &str) -> u32 {
        match multiplex_str.to_lowercase().as_str() {
            "direct" => 0,
            "stripe" => 1,
            "checkered" | "checker" => 2,
            "spiral" => 3,
            "zstripe" | "zstripe08" => 4,
            "znmirrorzstripe" => 5,
            "coreman" => 6,
            "kaler2scan" => 7,
            "zstripeuneven" => 8,
            "p10-128x4-z" => 9,
            "qiangliq8" => 10,
            "inversedzstripe" => 11,
            "p10outdoor1r1g1-1" => 12,
            "p10outdoor1r1g1-2" => 13,
            "p10outdoor1r1g1-3" => 14,
            "p10coremanmapper" => 15,
            "p8outdoor1r1g1" => 16,
            _ => {
                warn!("Unknown multiplexing type '{}' for binding driver, using default (Stripe)", multiplex_str);
                1 // Default to Stripe (1)
            }
        }
    }
    
    // Helper to map row setter string to numeric value 
    fn map_row_setter(row_setter: &str) -> u32 {
        match row_setter.to_lowercase().as_str() {
            "default" => 0,
            "ab-addressed" => 1,
            "direct" => 2,
            "abc-addressed" => 3,
            "abc-shift-de" => 4,
            _ => {
                warn!("Unknown row address setter '{}' for binding driver, using default", row_setter);
                0 // Default to 0 for unknown values
            }
        }
    }
} 