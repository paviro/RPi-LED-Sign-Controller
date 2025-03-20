use std::fmt::Debug;
use crate::config::DisplayConfig;

mod options;
mod rpi_led_panel_driver;
mod rpi_led_matrix_driver;

pub use rpi_led_panel_driver::RpiLedPanelDriver;
pub use rpi_led_matrix_driver::RpiLedMatrixDriver;

// Core traits
pub trait LedCanvas: Debug + Send {
    fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8);
    fn fill(&mut self, r: u8, g: u8, b: u8);
    fn size(&self) -> (i32, i32); // (width, height)
    
    // For downcasting - need a way to convert to specific implementation
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any where Self: 'static;
}

pub trait LedDriver: Debug + Send {
    fn initialize(config: &DisplayConfig) -> Result<Self, String> where Self: Sized;
    fn take_canvas(&mut self) -> Option<Box<dyn LedCanvas>>;
    fn update_canvas(&mut self, canvas: Box<dyn LedCanvas>) -> Box<dyn LedCanvas>;
    fn shutdown(&mut self);
}

// Enumeration of supported drivers
#[derive(Debug, Clone, PartialEq)]
pub enum DriverType {
    RpiLedPanel, // Native Rust driver
    RpiLedMatrix, // C++ binding driver
}

// Factory function to create the appropriate driver
pub fn create_driver(config: &DisplayConfig) -> Result<Box<dyn LedDriver>, String> {
    match config.driver_type {
        DriverType::RpiLedPanel => {
            log::debug!("Creating rpi-led-panel driver");
            Ok(Box::new(RpiLedPanelDriver::initialize(config)?))
        },
        DriverType::RpiLedMatrix => {
            log::debug!("Creating rpi-led-matrix driver");
            Ok(Box::new(RpiLedMatrixDriver::initialize(config)?))
        },
    }
} 