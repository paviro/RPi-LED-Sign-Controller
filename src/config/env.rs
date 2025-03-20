//! Environment variable handling

/// Environment variables for LED matrix configuration
#[derive(Debug, Default, Clone)]
pub struct EnvVars {
    pub driver: Option<String>,
    pub rows: Option<usize>,
    pub cols: Option<usize>,
    pub chain_length: Option<usize>,
    pub parallel: Option<usize>,
    pub hardware_mapping: Option<String>,
    pub gpio_slowdown: Option<u32>,
    pub pwm_bits: Option<u8>,
    pub pwm_lsb_nanoseconds: Option<u32>,
    pub pixel_mapper: Option<String>,
    pub multiplexing: Option<String>,
    pub pi_chip: Option<String>,
    pub interlaced: Option<bool>,
    pub dither_bits: Option<usize>,
    pub panel_type: Option<String>,
    pub row_setter: Option<String>,
    pub led_sequence: Option<String>,
    pub hardware_pulsing: Option<bool>,
    pub show_refresh: Option<bool>,
    pub inverse_colors: Option<bool>,
    pub limit_refresh_rate: Option<u32>,
    pub port: Option<u16>,
    pub interface: Option<String>,
    pub limit_max_brightness: Option<u8>,
}

/// Load configuration from environment variables
pub fn load_env_vars() -> EnvVars {
    let mut env = EnvVars::default();
    
    // Driver type
    if let Ok(value) = std::env::var("LED_DRIVER") {
        env.driver = Some(value);
    }
    
    // Matrix dimensions
    if let Ok(value) = std::env::var("LED_ROWS") {
        if let Ok(rows) = value.parse() {
            env.rows = Some(rows);
        }
    }
    
    if let Ok(value) = std::env::var("LED_COLS") {
        if let Ok(cols) = value.parse() {
            env.cols = Some(cols);
        }
    }
    
    if let Ok(value) = std::env::var("LED_CHAIN_LENGTH") {
        if let Ok(chain) = value.parse() {
            env.chain_length = Some(chain);
        }
    }
    
    if let Ok(value) = std::env::var("LED_PARALLEL") {
        if let Ok(parallel) = value.parse() {
            env.parallel = Some(parallel);
        }
    }
    
    // Hardware configuration
    if let Ok(value) = std::env::var("LED_HARDWARE_MAPPING") {
        env.hardware_mapping = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_GPIO_SLOWDOWN") {
        if let Ok(slowdown) = value.parse() {
            env.gpio_slowdown = Some(slowdown);
        }
    }
    
    // PWM settings
    if let Ok(value) = std::env::var("LED_PWM_BITS") {
        if let Ok(bits) = value.parse() {
            env.pwm_bits = Some(bits);
        }
    }
    
    if let Ok(value) = std::env::var("LED_PWM_LSB_NANOSECONDS") {
        if let Ok(ns) = value.parse() {
            env.pwm_lsb_nanoseconds = Some(ns);
        }
    }
    
    // Panel configuration
    if let Ok(value) = std::env::var("LED_PIXEL_MAPPER") {
        env.pixel_mapper = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_MULTIPLEXING") {
        env.multiplexing = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_PI_CHIP") {
        env.pi_chip = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_INTERLACED") {
        if let Ok(enabled) = value.parse::<bool>() {
            env.interlaced = Some(enabled);
        } else if let Ok(enabled) = value.parse::<u8>() {
            // Also support numeric values (0/1)
            env.interlaced = Some(enabled != 0);
        }
    }
    
    if let Ok(value) = std::env::var("LED_DITHER_BITS") {
        if let Ok(bits) = value.parse() {
            env.dither_bits = Some(bits);
        }
    }
    
    if let Ok(value) = std::env::var("LED_PANEL_TYPE") {
        env.panel_type = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_ROW_SETTER") {
        env.row_setter = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_SEQUENCE") {
        env.led_sequence = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_HARDWARE_PULSING") {
        if let Ok(enabled) = value.parse::<bool>() {
            env.hardware_pulsing = Some(enabled);
        } else if let Ok(enabled) = value.parse::<u8>() {
            // Also support numeric values (0/1)
            env.hardware_pulsing = Some(enabled != 0);
        }
    }
    
    if let Ok(value) = std::env::var("LED_SHOW_REFRESH") {
        if let Ok(enabled) = value.parse::<bool>() {
            env.show_refresh = Some(enabled);
        } else if let Ok(enabled) = value.parse::<u8>() {
            env.show_refresh = Some(enabled != 0);
        }
    }
    
    if let Ok(value) = std::env::var("LED_INVERSE_COLORS") {
        if let Ok(enabled) = value.parse::<bool>() {
            env.inverse_colors = Some(enabled);
        } else if let Ok(enabled) = value.parse::<u8>() {
            env.inverse_colors = Some(enabled != 0);
        }
    }
    
    if let Ok(value) = std::env::var("LED_LIMIT_REFRESH_RATE") {
        if let Ok(limit) = value.parse() {
            env.limit_refresh_rate = Some(limit);
        }
    }
    
    // Web server settings
    if let Ok(value) = std::env::var("LED_PORT") {
        if let Ok(port) = value.parse() {
            env.port = Some(port);
        }
    }
    
    if let Ok(value) = std::env::var("LED_INTERFACE") {
        env.interface = Some(value);
    }
    
    if let Ok(value) = std::env::var("LED_LIMIT_MAX_BRIGHTNESS") {
        if let Ok(brightness_limit) = value.parse::<u8>() {
            env.limit_max_brightness = Some(brightness_limit.clamp(0, 100));
        }
    }
    
    env
} 