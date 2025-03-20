use std::fmt::Debug;
use std::any::Any;
use log::{debug, warn};
use rpi_led_panel::{RGBMatrix, Canvas, HardwareMapping, LedSequence, 
                    PiChip, PanelType, MultiplexMapperType, RowAddressSetterType, RGBMatrixConfig};

use crate::config::DisplayConfig;
use super::{LedCanvas, LedDriver};
use super::options::MatrixOptions;

// Canvas implementation for rpi-led-panel
pub struct RpiLedPanelCanvas {
    canvas: Option<Box<Canvas>>,
    width: i32,
    height: i32,
}

// Manual Debug impl since Canvas doesn't implement Debug
impl Debug for RpiLedPanelCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpiLedPanelCanvas")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

// Explicitly implement Send for thread safety
unsafe impl Send for RpiLedPanelCanvas {}

impl LedCanvas for RpiLedPanelCanvas {
    fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if let Some(canvas) = &mut self.canvas {
            canvas.set_pixel(x, y, r, g, b);
        }
    }

    fn fill(&mut self, r: u8, g: u8, b: u8) {
        if let Some(canvas) = &mut self.canvas {
            canvas.fill(r, g, b);
        }
    }

    fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any where Self: 'static {
        self
    }
}

// Driver implementation for rpi-led-panel
pub struct RpiLedPanelDriver {
    matrix: RGBMatrix,
    canvas: Option<Box<Canvas>>,
    width: i32,
    height: i32,
}

// Manual Debug impl since RGBMatrix doesn't implement Debug
impl Debug for RpiLedPanelDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpiLedPanelDriver")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("has_canvas", &self.canvas.is_some())
            .finish()
    }
}

// Explicitly implement Send for thread safety
unsafe impl Send for RpiLedPanelDriver {}

impl LedDriver for RpiLedPanelDriver {
    fn initialize(config: &DisplayConfig) -> Result<Self, String> where Self: Sized {
        // Get common options
        let options = MatrixOptions::from_config(config);
        
        // Convert to rpi-led-panel specific config
        let matrix_config = Self::create_matrix_config(&options)?;
        
        debug!("Initializing rpi-led-panel with options: {:?}", options);
        
        match RGBMatrix::new(matrix_config, 0) {
            Ok((matrix, canvas)) => {
                let width = (options.cols * options.chain_length) as i32;
                let height = (options.rows * options.parallel) as i32;
                
                Ok(Self {
                    matrix,
                    canvas: Some(canvas),
                    width,
                    height,
                })
            },
            Err(e) => Err(format!("Failed to initialize rpi-led-panel: {}", e)),
        }
    }

    fn take_canvas(&mut self) -> Option<Box<dyn LedCanvas>> {
        if let Some(canvas) = self.canvas.take() {
            Some(Box::new(RpiLedPanelCanvas {
                canvas: Some(canvas),
                width: self.width,
                height: self.height,
            }))
        } else {
            None
        }
    }

    fn update_canvas(&mut self, mut canvas: Box<dyn LedCanvas>) -> Box<dyn LedCanvas> {
        // Get the specific canvas type using as_any
        let panel_canvas: &mut RpiLedPanelCanvas = canvas
            .as_any_mut()
            .downcast_mut::<RpiLedPanelCanvas>()
            .expect("Canvas was not an RpiLedPanelCanvas");
        
        // Extract dimensions for reuse
        let width = panel_canvas.width;
        let height = panel_canvas.height;
        
        // Extract the canvas directly
        let inner_canvas = panel_canvas.canvas.take()
            .expect("Canvas was None when it shouldn't be");
        
        // Update the display and get the new canvas
        let new_canvas = self.matrix.update_on_vsync(inner_canvas);
        
        // Wrap in our canvas type
        Box::new(RpiLedPanelCanvas {
            canvas: Some(new_canvas),
            width,
            height,
        })
    }

    fn shutdown(&mut self) {
        // For the panel driver, create a black canvas and update it
        if let Some(mut canvas) = self.canvas.take() {
            canvas.fill(0, 0, 0); // Fill with black
            let _ = self.matrix.update_on_vsync(canvas); // Update one last time
        }
    }
}

impl RpiLedPanelDriver {
    // Helper method to create native driver config
    fn create_matrix_config(options: &MatrixOptions) -> Result<RGBMatrixConfig, String> {
        let mut config = RGBMatrixConfig::default();
        let mut unsupported_options = Vec::new();
        
        // Set basic options
        config.rows = options.rows;
        config.cols = options.cols;
        config.chain_length = options.chain_length;
        config.parallel = options.parallel;
        config.led_brightness = options.brightness;
        config.refresh_rate = 120; // Set a default refresh rate that's reasonable
        
        // Set additional options
        config.pwm_bits = options.pwm_bits as usize;
        config.pwm_lsb_nanoseconds = options.pwm_lsb_nanoseconds;
        config.interlaced = options.interlaced;
        config.dither_bits = options.dither_bits;
        
        // Convert hardware mapping
        config.hardware_mapping = match options.hardware_mapping.to_lowercase().as_str() {
            "regular" => HardwareMapping::regular(),
            "adafruit-hat" | "adafruithat" => HardwareMapping::adafruit_hat(),
            "adafruit-hat-pwm" | "adafruithatpwm" => HardwareMapping::adafruit_hat_pwm(),
            "regular-pi1" => HardwareMapping::regular_pi1(),
            "classic" => HardwareMapping::classic(),
            "classic-pi1" => HardwareMapping::classic_pi1(),
            mapping => {
                warn!("Unsupported hardware mapping '{}' for native driver, defaulting to 'regular'", mapping);
                unsupported_options.push(format!("hardware_mapping={}", mapping));
                HardwareMapping::regular()
            }
        };
        
        // Convert LED sequence
        config.led_sequence = match options.led_sequence.to_uppercase().as_str() {
            "RGB" => LedSequence::Rgb,
            "RBG" => LedSequence::Rbg,
            "GRB" => LedSequence::Grb,
            "GBR" => LedSequence::Gbr,
            "BRG" => LedSequence::Brg,
            "BGR" => LedSequence::Bgr,
            seq => {
                warn!("Unsupported LED sequence '{}' for native driver, defaulting to 'RGB'", seq);
                unsupported_options.push(format!("led_sequence={}", seq));
                LedSequence::Rgb
            }
        };
        
        // Apply Pi chip if specified
        if let Some(chip) = &options.pi_chip {
            config.pi_chip = match chip.to_uppercase().as_str() {
                "BCM2708" => Some(PiChip::BCM2708), // Pi 1
                "BCM2709" => Some(PiChip::BCM2709), // Pi 2
                "BCM2711" => Some(PiChip::BCM2711), // Pi 4
                chip_type => {
                    warn!("Unsupported Pi chip '{}' for native driver, using automatic detection", chip_type);
                    unsupported_options.push(format!("pi_chip={}", chip_type));
                    None
                }
            };
        }
        
        // Apply panel type if specified
        if let Some(panel) = &options.panel_type {
            config.panel_type = match panel.to_uppercase().as_str() {
                "FM6126" | "FM6126A" => Some(PanelType::FM6126),
                "FM6127" => Some(PanelType::FM6127),
                panel_type => {
                    warn!("Unsupported panel type '{}' for native driver, using default", panel_type);
                    unsupported_options.push(format!("panel_type={}", panel_type));
                    None
                }
            };
        }
        
        // Apply multiplexing if specified
        if let Some(multiplex_str) = &options.multiplexing {
            let multiplex_type = Self::map_multiplexing(multiplex_str);
            if multiplex_type.is_none() {
                unsupported_options.push(format!("multiplexing={}", multiplex_str));
            }
            config.multiplexing = multiplex_type;
        }
        
        // Convert row address setter
        config.row_setter = Self::map_row_setter(&options.row_setter);
        
        // Apply pixel mapper if specified - just warn and add to unsupported options
        if let Some(mappers) = &options.pixel_mapper {
            warn!("Pixel mapper '{}' not directly supported in the native driver", mappers);
            unsupported_options.push(format!("pixel_mapper={}", mappers));
        }
        
        // Set GPIO slowdown if specified
        if let Some(slowdown) = options.gpio_slowdown {
            config.slowdown = Some(slowdown);
        }
        
        // Check for unsupported options
        if !options.hardware_pulsing {
            unsupported_options.push("no-hardware-pulse".to_string());
        }

        if options.show_refresh {
            unsupported_options.push("show-refresh".to_string());
        }

        if options.inverse_colors {
            unsupported_options.push("inverse-colors".to_string());
        }

        if options.limit_refresh > 0 {
            unsupported_options.push(format!("limit-refresh={}", options.limit_refresh));
        }
        
        // Check if we encountered any unsupported options
        if !unsupported_options.is_empty() {
            return Err(format!(
                "The following options are not supported by the native driver: {}",
                unsupported_options.join(", ")
            ));
        }
        
        Ok(config)
    }
    
    // Helper to map multiplexing strings to enum values
    fn map_multiplexing(multiplex_str: &str) -> Option<MultiplexMapperType> {
        let multiplex_str = multiplex_str.to_lowercase();
        
        match multiplex_str.as_str() {
            "stripe" => Some(MultiplexMapperType::Stripe),
            "checkered" | "checker" => Some(MultiplexMapperType::Checkered),
            "spiral" => Some(MultiplexMapperType::Spiral),
            "zstripe" | "zstripe08" => Some(MultiplexMapperType::ZStripe08),
            "zstripe44" => Some(MultiplexMapperType::ZStripe44),
            "zstripe80" => Some(MultiplexMapperType::ZStripe80),
            "coreman" => Some(MultiplexMapperType::Coreman),
            "kaler2scan" => Some(MultiplexMapperType::Kaler2Scan),
            "p10z" => Some(MultiplexMapperType::P10Z),
            "qiangliq8" => Some(MultiplexMapperType::QiangLiQ8),
            "inversedzstripe" => Some(MultiplexMapperType::InversedZStripe),
            "p10outdoor1r1g1b1" => Some(MultiplexMapperType::P10Outdoor1R1G1B1),
            "p10outdoor1r1g1b2" => Some(MultiplexMapperType::P10Outdoor1R1G1B2),
            "p10outdoor1r1g1b3" => Some(MultiplexMapperType::P10Outdoor1R1G1B3),
            "p10coreman" => Some(MultiplexMapperType::P10Coreman),
            "p8outdoor1r1g1b" => Some(MultiplexMapperType::P8Outdoor1R1G1B),
            "flippedstripe" => Some(MultiplexMapperType::FlippedStripe),
            "p10outdoor32x16halfscan" => Some(MultiplexMapperType::P10Outdoor32x16HalfScan),
            _ => None
        }
    }
    
    // Helper to map row setter strings to enum values
    fn map_row_setter(row_setter: &str) -> RowAddressSetterType {
        match row_setter.to_lowercase().as_str() {
            "direct" => RowAddressSetterType::Direct,
            "shift-register" | "shiftregister" => RowAddressSetterType::ShiftRegister,
            "direct-abcd" | "directabcdline" => RowAddressSetterType::DirectABCDLine,
            "abc-shift-register" | "abcshiftregister" => RowAddressSetterType::ABCShiftRegister,
            "sm5266" => RowAddressSetterType::SM5266,
            _ => {
                warn!("Unknown row address setter '{}', using direct", row_setter);
                RowAddressSetterType::Direct
            }
        }
    }
} 