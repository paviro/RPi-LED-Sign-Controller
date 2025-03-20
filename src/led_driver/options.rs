use crate::config::DisplayConfig;

// Common options for both drivers
#[derive(Debug, Clone)]
pub struct MatrixOptions {
    // Basic display options
    pub rows: usize,
    pub cols: usize,
    pub chain_length: usize,
    pub parallel: usize,
    pub brightness: u8,
    
    // Additional options
    pub hardware_mapping: String,
    pub pwm_bits: u8,
    pub pwm_lsb_nanoseconds: u32,
    pub gpio_slowdown: Option<u32>,
    pub interlaced: bool,
    pub dither_bits: usize,
    pub panel_type: Option<String>,
    pub multiplexing: Option<String>,
    pub pixel_mapper: Option<String>,
    pub row_setter: String,
    pub led_sequence: String,
    
    // New C++ binding specific options
    pub hardware_pulsing: bool,
    pub show_refresh: bool,
    pub inverse_colors: bool,
    pub limit_refresh: u32,
    pub pi_chip: Option<String>,
}

impl Default for MatrixOptions {
    fn default() -> Self {
        Self {
            rows: 32,
            cols: 64,
            chain_length: 1,
            parallel: 1,
            brightness: 100,
            hardware_mapping: "regular".to_string(),
            pwm_bits: 11,
            pwm_lsb_nanoseconds: 130,
            gpio_slowdown: None,
            interlaced: false,
            dither_bits: 0,
            panel_type: None,
            multiplexing: None,
            pixel_mapper: None,
            row_setter: "default".to_string(),
            led_sequence: "RGB".to_string(),
            hardware_pulsing: true,
            show_refresh: false,
            inverse_colors: false,
            limit_refresh: 0,
            pi_chip: None,
        }
    }
}

impl MatrixOptions {
    // Create from DisplayConfig
    pub fn from_config(config: &DisplayConfig) -> Self {
        let mut options = Self {
            rows: config.rows,
            cols: config.cols,
            chain_length: config.chain_length,
            parallel: config.parallel,
            brightness: config.led_brightness,
            // Apply CLI arguments
            hardware_mapping: config.hardware_mapping.clone(),
            pwm_bits: config.pwm_bits,
            pwm_lsb_nanoseconds: config.pwm_lsb_nanoseconds,
            gpio_slowdown: config.gpio_slowdown,
            interlaced: config.interlaced,
            dither_bits: config.dither_bits,
            panel_type: config.panel_type.clone(),
            multiplexing: config.multiplexing.clone(),
            pixel_mapper: config.pixel_mapper.clone(),
            row_setter: config.row_setter.clone(),
            led_sequence: config.led_sequence.clone(),
            hardware_pulsing: config.hardware_pulsing,
            show_refresh: config.show_refresh,
            inverse_colors: config.inverse_colors,
            limit_refresh: config.limit_refresh,
            pi_chip: config.pi_chip.clone(),
        };
        
        // Apply any environment variable overrides
        Self::apply_env_overrides(&mut options);
        
        options
    }
    
    // Apply environment variable overrides
    fn apply_env_overrides(options: &mut Self) {
        // Matrix dimensions
        if let Ok(value) = std::env::var("LED_ROWS") {
            if let Ok(rows) = value.parse() {
                options.rows = rows;
            }
        }
        
        if let Ok(value) = std::env::var("LED_COLS") {
            if let Ok(cols) = value.parse() {
                options.cols = cols;
            }
        }
        
        if let Ok(value) = std::env::var("LED_CHAIN_LENGTH") {
            if let Ok(chain) = value.parse() {
                options.chain_length = chain;
            }
        }
        
        if let Ok(value) = std::env::var("LED_PARALLEL") {
            if let Ok(parallel) = value.parse() {
                options.parallel = parallel;
            }
        }
        
        if let Ok(value) = std::env::var("LED_BRIGHTNESS") {
            if let Ok(brightness) = value.parse::<u8>() {
                options.brightness = brightness.clamp(0, 100);
            }
        }
        
        // Hardware configuration
        if let Ok(mapping) = std::env::var("LED_HARDWARE_MAPPING") {
            options.hardware_mapping = mapping;
        }
        
        if let Ok(slowdown) = std::env::var("LED_GPIO_SLOWDOWN") {
            if let Ok(val) = slowdown.parse::<u32>() {
                options.gpio_slowdown = Some(val);
            }
        }
        
        // PWM settings
        if let Ok(bits) = std::env::var("LED_PWM_BITS") {
            if let Ok(val) = bits.parse::<u8>() {
                options.pwm_bits = val;
            }
        }
        
        if let Ok(ns) = std::env::var("LED_PWM_LSB_NANOSECONDS") {
            if let Ok(val) = ns.parse::<u32>() {
                options.pwm_lsb_nanoseconds = val;
            }
        }
        
        // Panel configuration
        if let Ok(mapper) = std::env::var("LED_PIXEL_MAPPER") {
            options.pixel_mapper = Some(mapper);
        }
        
        if let Ok(multiplex) = std::env::var("LED_MULTIPLEXING") {
            options.multiplexing = Some(multiplex);
        }
        
        if let Ok(value) = std::env::var("LED_PANEL_TYPE") {
            options.panel_type = Some(value);
        }
        
        if let Ok(_value) = std::env::var("LED_PI_CHIP") {
            // We don't use this directly in MatrixOptions, 
            // but it's passed to the driver implementations
        }
        
        if let Ok(value) = std::env::var("LED_INTERLACED") {
            if let Ok(enabled) = value.parse::<bool>() {
                options.interlaced = enabled;
            } else if let Ok(enabled) = value.parse::<u8>() {
                // Also support numeric values (0/1)
                options.interlaced = enabled != 0;
            }
        }
        
        if let Ok(value) = std::env::var("LED_DITHER_BITS") {
            if let Ok(bits) = value.parse() {
                options.dither_bits = bits;
            }
        }
        
        if let Ok(value) = std::env::var("LED_ROW_SETTER") {
            options.row_setter = value;
        }
        
        if let Ok(value) = std::env::var("LED_SEQUENCE") {
            options.led_sequence = value;
        }
        
        if let Ok(value) = std::env::var("LED_HARDWARE_PULSING") {
            if let Ok(enabled) = value.parse::<bool>() {
                options.hardware_pulsing = enabled;
            } else if let Ok(enabled) = value.parse::<u8>() {
                options.hardware_pulsing = enabled != 0;
            }
        }
        
        if let Ok(value) = std::env::var("LED_SHOW_REFRESH") {
            if let Ok(enabled) = value.parse::<bool>() {
                options.show_refresh = enabled;
            } else if let Ok(enabled) = value.parse::<u8>() {
                options.show_refresh = enabled != 0;
            }
        }
        
        if let Ok(value) = std::env::var("LED_INVERSE_COLORS") {
            if let Ok(enabled) = value.parse::<bool>() {
                options.inverse_colors = enabled;
            } else if let Ok(enabled) = value.parse::<u8>() {
                options.inverse_colors = enabled != 0;
            }
        }
        
        if let Ok(value) = std::env::var("LED_LIMIT_REFRESH") {
            if let Ok(limit) = value.parse() {
                options.limit_refresh = limit;
            }
        }
    }
} 