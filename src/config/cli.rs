//! Command-line argument parsing

/// Command-line arguments for the LED Matrix Display Controller
#[derive(argh::FromArgs, Debug, Clone)]
/// LED Matrix Display Controller
///
/// Controls an LED matrix display with web configuration interface.
pub struct CliArgs {
    #[argh(option, short = 'd')]
    /// driver type: "native" or "binding"
    /// 
    /// native: Pure Rust library (https://github.com/EmbersArc/rpi_led_panel)
    /// binding: C++ binding (https://github.com/hzeller/rpi-rgb-led-matrix)
    /// 
    /// (REQUIRED)
    pub driver: Option<String>,

    #[argh(option, short = 'r', default = "32")]
    /// number of rows. Default: 32 [native, binding]
    pub rows: usize,
    
    #[argh(option, short = 'c', default = "64")]
    /// number of columns. Default: 64 [native, binding]
    pub cols: usize,

    #[argh(option, short = 'p', default = "1")]
    /// how many chains to run in parallel. Default: 1 [native, binding]
    /// note: both drivers only support values 1-3
    pub parallel: usize,

    #[argh(option, short = 'n', default = "1")]
    /// number of daisy-chained panels. Default: 1 [native, binding]
    pub chain_length: usize,

    #[argh(option)]
    /// the display wiring e.g. "regular", "adafruit-hat", or "adafruit-hat-pwm".
    /// Default: "regular" [native, binding]
    pub hardware_mapping: Option<String>,
    
    #[argh(option)]
    /// the Raspberry Pi chip model e.g. "BCM2711".
    /// Default: automatic [native]
    pub pi_chip: Option<String>,
    
    #[argh(option, default = "11")]
    /// PWM bits for color depth control (1-11). Default: 11 [native, binding]
    pub pwm_bits: u8,
    
    #[argh(option, default = "130")]
    /// base time-unit for the on-time in the lowest significant bit in nanoseconds.
    /// Default: 130 [native, binding]
    pub pwm_lsb_nanoseconds: u32,
    
    #[argh(option)]
    /// GPIO slowdown factor (0-4). Default: automatic based on Pi model [native, binding]
    pub gpio_slowdown: Option<u32>,
    
    #[argh(switch)]
    /// enable interlaced scan mode. Default: false [native, binding]
    pub interlaced: bool,
    
    #[argh(option, default = "0")]
    /// number of bits to use for time dithering. Default: 0 (no dithering) [native, binding]
    pub dither_bits: usize,
    
    #[argh(option)]
    /// panel type, e.g. "FM6126A" for panels requiring special initialization [native, binding]
    pub panel_type: Option<String>,
    
    #[argh(option)]
    /// multiplexing type (e.g., "Stripe", "Checkered", "Spiral", etc.) [native, binding]
    pub multiplexing: Option<String>,
    
    #[argh(option)]
    /// semicolon-separated list of pixel-mappers to arrange pixels
    /// (e.g. "U-mapper;Rotate:90") [native, binding]
    pub pixel_mapper: Option<String>,
    
    #[argh(option, default = "String::from(\"direct\")")]
    /// row address setter type. Default: "direct" [native, binding]
    /// Valid options: "direct"/"default", "shiftregister"/"ab-addressed", 
    /// "directabcdline"/"direct-row-select", "abcshiftregister"/"abc-addressed",
    /// "sm5266"/"abc-shift-de"
    pub row_setter: String,
    
    #[argh(option, default = "String::from(\"RGB\")")]
    /// the LED color sequence, Default: "RGB" [native, binding]
    pub led_sequence: String,

    #[argh(switch)]
    /// disable hardware pin-pulse generation. Default: false (hardware pulse enabled) [binding]
    pub no_hardware_pulse: bool,

    #[argh(switch)]
    /// show refresh rate statistics on the terminal. Default: false [binding]
    pub show_refresh: bool,

    #[argh(switch)]
    /// invert display colors. Default: false [binding]
    pub inverse_colors: bool,

    #[argh(option, default = "0")]
    /// limit refresh rate in Hz (0 = no limit)
    /// Default: 0 (unlimited) [native, binding]
    pub limit_refresh_rate: u32,

    #[argh(option, default = "3000")]
    /// web server port. Default: 3000
    pub port: u16,
    
    #[argh(option, default = "String::from(\"0.0.0.0\")")]
    /// network interface to bind to. Default: "0.0.0.0" (all interfaces)
    pub interface: String,

    #[argh(option, default = "100")]
    /// maximum brightness limit (0-100). The UI's 100% setting will equal this value.
    /// Default: 100 (no scaling)
    pub limit_max_brightness: u8,
}

impl CliArgs {
    /// Parse CLI arguments
    pub fn parse() -> Self {
        // Use argh to parse args from environment
        argh::from_env()
    }
} 