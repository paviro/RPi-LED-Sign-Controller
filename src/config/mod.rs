//! Configuration module that handles all application settings

mod cli;
mod display;
mod env;

pub use cli::CliArgs;
pub use display::DisplayConfig;
pub use env::{load_env_vars, EnvVars};

/// Initialize configuration from all sources (CLI, environment, etc.)
pub fn init_config() -> DisplayConfig {
    // Parse CLI args first
    let cli_args = CliArgs::parse();

    // Load environment variables
    let env_vars = load_env_vars();

    // Create DisplayConfig by combining CLI args and environment variables
    DisplayConfig::new(cli_args, env_vars)
}
