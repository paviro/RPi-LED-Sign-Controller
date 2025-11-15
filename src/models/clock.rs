use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClockFormat {
    #[serde(rename = "24h")]
    TwentyFourHour,
    #[serde(rename = "12h")]
    TwelveHour,
}

impl Default for ClockFormat {
    fn default() -> Self {
        ClockFormat::TwentyFourHour
    }
}

fn default_show_seconds() -> bool {
    false
}

fn default_clock_color() -> [u8; 3] {
    [255, 255, 255]
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClockContent {
    #[serde(default)]
    pub format: ClockFormat,
    #[serde(default = "default_show_seconds")]
    pub show_seconds: bool,
    #[serde(default = "default_clock_color")]
    pub color: [u8; 3],
}
