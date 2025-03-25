use serde::{Deserialize, Serialize, ser::{Serializer, SerializeMap}};

// Border effects enum
#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum BorderEffect {
    None,
    Rainbow,
    Pulse { colors: Vec<[u8; 3]> },
    Sparkle { colors: Vec<[u8; 3]> },
    Gradient { colors: Vec<[u8; 3]> },
}

// Provide defaults
impl Default for BorderEffect {
    fn default() -> Self {
        BorderEffect::None
    }
}

impl Serialize for BorderEffect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Simple variants get serialized as {"Variant": null}
            BorderEffect::None => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("None", &Option::<()>::None)?;
                map.end()
            },
            BorderEffect::Rainbow => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Rainbow", &Option::<()>::None)?;
                map.end()
            },
            // Complex variants continue using the default serialization
            BorderEffect::Pulse { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Pulse", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
            BorderEffect::Sparkle { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Sparkle", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
            BorderEffect::Gradient { colors } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Gradient", &serde_json::json!({"colors": colors}))?;
                map.end()
            },
        }
    }
} 