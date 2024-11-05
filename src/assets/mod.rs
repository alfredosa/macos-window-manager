use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/assets/"] // Point to your current assets location
pub struct Asset;
