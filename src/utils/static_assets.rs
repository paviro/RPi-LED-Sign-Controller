use rust_embed::RustEmbed;

#[derive(RustEmbed, Clone)]
#[folder = "static/"]
pub struct StaticAssets; 