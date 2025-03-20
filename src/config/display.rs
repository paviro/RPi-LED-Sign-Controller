//! Display configuration structure and methods

use crate::led_driver::DriverType;
use log::info;
use super::{CliArgs, EnvVars};

/// Configuration structure that stores all display settings
#[derive(Clone, Debug)]
pub struct DisplayConfig {
    pub rows: usize,           
    pub cols: usize,           
    pub chain_length: usize,   
    pub parallel: usize,       
    pub led_brightness: u8,
    pub driver_type: DriverType,
    
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
    #[allow(dead_code)]
    pub pi_chip: Option<String>,
    pub hardware_pulsing: bool,
    pub show_refresh: bool,
    pub inverse_colors: bool,
    pub limit_refresh_rate: u32,
    pub limit_max_brightness: u8,
    
    // Web server configuration
    pub port: u16,
    pub interface: String,
}

impl DisplayConfig {
    /// Create a new configuration by combining CLI arguments and environment variables
    pub fn new(cli_args: CliArgs, env_vars: EnvVars) -> Self {
        // Determine driver type from CLI argument or environment variable
        let driver_arg = env_vars.driver.or_else(|| cli_args.driver.clone());
        
        let driver_type = match &driver_arg {
            Some(driver) if driver == "binding" => {
                info!("Selected driver: C++ binding for rpi-rgb-led-matrix (@https://github.com/hzeller/rpi-rgb-led-matrix)");
                DriverType::RpiLedMatrix
            },
            Some(driver) if driver == "native" => {
                info!("Selected driver: Native library rpi_led_panel (@https://github.com/EmbersArc/rpi_led_panel)");
                DriverType::RpiLedPanel
            },
            None => {
                println!("ERROR: You must specify a driver type (--driver native|binding or LED_DRIVER=native|binding)");
                println!("\nFor help, run: {} --help", std::env::args().next().unwrap_or_else(|| "program".to_string()));
                std::process::exit(1);
            },
            _ => {
                println!("ERROR: Invalid driver type: {:?}. Must be 'native' or 'binding'", driver_arg);
                println!("\nFor help, run: {} --help", std::env::args().next().unwrap_or_else(|| "program".to_string()));
                std::process::exit(1);
            }
        };
        
        // Apply settings from CLI args, then override with environment variables if present
        let rows = env_vars.rows.unwrap_or(cli_args.rows);
        let cols = env_vars.cols.unwrap_or(cli_args.cols);
        let chain_length = env_vars.chain_length.unwrap_or(cli_args.chain_length);
        let parallel = env_vars.parallel.unwrap_or(cli_args.parallel);
        
        let limit_max_brightness = env_vars.limit_max_brightness
            .unwrap_or(cli_args.limit_max_brightness)
            .clamp(0, 100);

        let led_brightness = limit_max_brightness;
        
        // Hardware settings
        let hardware_mapping = env_vars.hardware_mapping
            .unwrap_or_else(|| cli_args.hardware_mapping.unwrap_or_else(|| "regular".to_string()));
        
        // PWM settings
        let pwm_bits = env_vars.pwm_bits
            .unwrap_or(cli_args.pwm_bits)
            .clamp(1, 11);
            
        let pwm_lsb_nanoseconds = env_vars.pwm_lsb_nanoseconds
            .unwrap_or(cli_args.pwm_lsb_nanoseconds);
        
        // GPU slowdown
        let gpio_slowdown = env_vars.gpio_slowdown.or(cli_args.gpio_slowdown);
        
        // Panel configuration
        let multiplexing = env_vars.multiplexing.or(cli_args.multiplexing);
        let pixel_mapper = env_vars.pixel_mapper.or(cli_args.pixel_mapper);
        
        // Other settings from environment variables
        let limit_refresh_rate = env_vars.limit_refresh_rate.unwrap_or(cli_args.limit_refresh_rate);
        let interlaced = env_vars.interlaced.unwrap_or(cli_args.interlaced);
        let dither_bits = env_vars.dither_bits.unwrap_or(cli_args.dither_bits);
        let panel_type = env_vars.panel_type.or(cli_args.panel_type);
        let row_setter = env_vars.row_setter.unwrap_or_else(|| cli_args.row_setter);
        let led_sequence = env_vars.led_sequence.unwrap_or_else(|| cli_args.led_sequence);
        let pi_chip = env_vars.pi_chip.or(cli_args.pi_chip);
        
        let hardware_pulsing = env_vars.hardware_pulsing.unwrap_or(!cli_args.no_hardware_pulse);
        let show_refresh = env_vars.show_refresh.unwrap_or(cli_args.show_refresh);
        let inverse_colors = env_vars.inverse_colors.unwrap_or(cli_args.inverse_colors);
        
        // Web server settings
        let port = env_vars.port.unwrap_or(cli_args.port);
        
        let interface = env_vars.interface
            .unwrap_or_else(|| cli_args.interface)
            .to_lowercase();
        
        let interface = if interface == "localhost" {
            "127.0.0.1".to_string()
        } else {
            interface
        };
        
        Self {
            rows,
            cols,
            chain_length,
            parallel,
            led_brightness,
            driver_type,
            
            hardware_mapping,
            pwm_bits,
            pwm_lsb_nanoseconds,
            gpio_slowdown,
            interlaced,
            dither_bits,
            panel_type,
            multiplexing,
            pixel_mapper,
            row_setter,
            led_sequence,
            pi_chip,
            hardware_pulsing,
            show_refresh,
            inverse_colors,
            limit_refresh_rate,
            limit_max_brightness,
            port,
            interface,
        }
    }
    
    /// Calculate the total display width in pixels
    pub fn display_width(&self) -> i32 {
        (self.cols * self.chain_length) as i32
    }
    
    /// Calculate the total display height in pixels
    pub fn display_height(&self) -> i32 {
        (self.rows * self.parallel) as i32
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.rows == 0 {
            errors.push("Rows must be greater than 0".to_string());
        }
        
        if self.cols == 0 {
            errors.push("Columns must be greater than 0".to_string());
        }
        
        if self.chain_length == 0 {
            errors.push("Chain length must be greater than 0".to_string());
        }
        
        if self.parallel == 0 {
            errors.push("Parallel chains must be greater than 0".to_string());
        }
        
        if self.parallel > 3 {
            errors.push("Parallel chains must be between 1 and 3 (limitation of both drivers)".to_string());
        }
        
        if self.pwm_bits < 1 || self.pwm_bits > 11 {
            errors.push("PWM bits must be between 1 and 11".to_string());
        }
        
        if self.led_brightness > 100 {
            errors.push("LED brightness must be between 0 and 100".to_string());
        }
        
        if let Some(slowdown) = self.gpio_slowdown {
            if slowdown > 4 {
                errors.push("GPIO slowdown must be between 0 and 4".to_string());
            }
        }
        
        if let Err(e) = self.interface.parse::<std::net::IpAddr>() {
            errors.push(format!("Invalid network interface address '{}': {}. Use a valid IP address or 'localhost'", 
                self.interface, e));
        }
        
        if self.limit_max_brightness > 100 {
            errors.push("Maximum brightness limit must be between 0 and 100".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
} 