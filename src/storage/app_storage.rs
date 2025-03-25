use std::sync::{Arc, Mutex};
use crate::models::playlist::Playlist;
use crate::storage::manager::{StorageManager, paths};
use log::{info, error, debug};

// Unified storage for all application settings
pub struct AppStorage {
    storage_manager: StorageManager,
}

impl AppStorage {
    pub fn new(storage_manager: StorageManager) -> Self {
        Self { storage_manager }
    }

    // Playlist-related methods
    pub fn load_playlist(&self) -> Option<Playlist> {
        // Check if the file exists first
        if !self.storage_manager.file_exists(paths::PLAYLIST_FILE) {
            debug!("No playlist file found");
            return None;
        }

        // Try to read and parse the file
        match self.storage_manager.read_file(paths::PLAYLIST_FILE) {
            Ok(contents) => {
                debug!("Loaded playlist file, attempting to parse");
                match serde_json::from_str::<Playlist>(&contents) {
                    Ok(playlist) => {
                        info!("Successfully loaded playlist with {} items", playlist.items.len());
                        if let Some(mut playlist) = Some(playlist) {
                            playlist.active_index = 0;
                            Some(playlist)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        error!("Error parsing playlist file: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error reading playlist file: {}", e);
                None
            }
        }
    }

    pub fn save_playlist(&self, playlist: &Playlist) -> bool {
        debug!("Saving playlist with {} items", playlist.items.len());
        
        // Serialize the playlist to JSON
        match serde_json::to_string_pretty(playlist) {
            Ok(json) => {
                // Write the JSON to the file
                match self.storage_manager.write_file(paths::PLAYLIST_FILE, &json) {
                    Ok(_) => {
                        let file_path = self.storage_manager.get_file_path(paths::PLAYLIST_FILE);
                        info!("Playlist saved to: {:?}", file_path);
                        true
                    }
                    Err(e) => {
                        error!("Error writing playlist file: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("Error serializing playlist: {}", e);
                false
            }
        }
    }
    
    // Display settings methods
    pub fn load_brightness(&self) -> Option<u8> {
        debug!("Loading brightness setting");
        
        if !self.storage_manager.file_exists(paths::BRIGHTNESS_FILE) {
            debug!("No brightness file found");
            return None;
        }
        
        match self.storage_manager.read_file(paths::BRIGHTNESS_FILE) {
            Ok(contents) => {
                #[derive(serde::Deserialize)]
                struct BrightnessSetting {
                    brightness: u8,
                }
                
                match serde_json::from_str::<BrightnessSetting>(&contents) {
                    Ok(setting) => {
                        info!("Loaded brightness setting: {}%", setting.brightness);
                        Some(setting.brightness)
                    }
                    Err(e) => {
                        error!("Error parsing brightness file: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error reading brightness file: {}", e);
                None
            }
        }
    }
    
    pub fn save_brightness(&self, brightness: u8) {
        debug!("Saving brightness setting: {}%", brightness);
        
        #[derive(serde::Serialize)]
        struct BrightnessSetting {
            brightness: u8,
        }
        
        let setting = BrightnessSetting { brightness };
        
        match serde_json::to_string_pretty(&setting) {
            Ok(json) => {
                match self.storage_manager.write_file(paths::BRIGHTNESS_FILE, &json) {
                    Ok(_) => {
                        info!("Brightness saved: {}%", brightness);
                    }
                    Err(e) => {
                        error!("Error writing brightness file: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error serializing brightness: {}", e);
            }
        }
    }
}

// Create a global storage instance that can be shared across threads
pub type SharedStorage = Arc<Mutex<AppStorage>>;

pub fn create_storage(custom_dir: Option<String>) -> SharedStorage {
    // Create the storage manager with the specified directory
    let storage_manager = StorageManager::new(custom_dir);
    
    // Create the app storage using the manager
    let app_storage = AppStorage::new(storage_manager);
    
    // Wrap in Arc<Mutex<>> for thread safety
    Arc::new(Mutex::new(app_storage))
} 