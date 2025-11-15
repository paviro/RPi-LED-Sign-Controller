use crate::models::content::ContentDetails;
use crate::models::playlist::Playlist;
use crate::storage::manager::{paths, StorageManager};
use log::{debug, error, info};
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

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
                        info!(
                            "Successfully loaded playlist with {} items",
                            playlist.items.len()
                        );
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
                match self
                    .storage_manager
                    .write_file(paths::BRIGHTNESS_FILE, &json)
                {
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

    // Image helpers
    pub fn save_image(&self, image_id: &str, data: &[u8]) -> bool {
        match self.storage_manager.save_image_file(image_id, data) {
            Ok(path) => {
                info!("Saved image {} to {:?}", image_id, path);
                true
            }
            Err(err) => {
                error!("Failed to save image {}: {}", image_id, err);
                false
            }
        }
    }

    pub fn save_thumbnail(&self, image_id: &str, data: &[u8]) -> bool {
        match self.storage_manager.save_thumbnail_file(image_id, data) {
            Ok(path) => {
                info!("Saved thumbnail {} to {:?}", image_id, path);
                true
            }
            Err(err) => {
                error!("Failed to save thumbnail {}: {}", image_id, err);
                false
            }
        }
    }

    pub fn load_image(&self, image_id: &str) -> Option<Vec<u8>> {
        match self.storage_manager.read_image_file(image_id) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                error!("Failed to read image {}: {}", image_id, err);
                None
            }
        }
    }

    pub fn load_thumbnail(&self, image_id: &str) -> Option<Vec<u8>> {
        match self.storage_manager.read_thumbnail_file(image_id) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                debug!(
                    "Failed to read thumbnail {}, will attempt regeneration if needed: {}",
                    image_id, err
                );
                None
            }
        }
    }

    pub fn image_path(&self, image_id: &str) -> std::path::PathBuf {
        self.storage_manager.image_file_path(image_id)
    }

    pub fn cleanup_unused_images(&self, playlist: &Playlist) -> usize {
        let referenced_ids: HashSet<String> = playlist
            .items
            .iter()
            .filter_map(|item| match &item.content.data {
                ContentDetails::Image(image_content) => Some(image_content.image_id.clone()),
                _ => None,
            })
            .collect();

        if let Err(err) = self.storage_manager.ensure_images_dir() {
            error!("Unable to ensure images directory before cleanup: {}", err);
            return 0;
        }

        let images_dir = self.storage_manager.get_file_path(paths::IMAGES_DIR);

        let dir_entries = match fs::read_dir(&images_dir) {
            Ok(entries) => entries,
            Err(err) => {
                debug!(
                    "Skipping image cleanup; could not read {:?}: {}",
                    images_dir, err
                );
                return 0;
            }
        };

        let mut removed_images = 0usize;
        let mut removed_thumbnails = 0usize;

        for entry in dir_entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    debug!("Failed to inspect image directory entry: {}", err);
                    continue;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let is_png = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false);

            if !is_png {
                continue;
            }

            let image_id = match path.file_stem().and_then(|stem| stem.to_str()) {
                Some(stem) => stem,
                None => continue,
            };

            if referenced_ids.contains(image_id) {
                continue;
            }

            let mut image_deleted = false;
            match fs::remove_file(&path) {
                Ok(_) => {
                    debug!("Removed unused image {}", image_id);
                    removed_images += 1;
                    image_deleted = true;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        debug!(
                            "Image {} already removed on disk during cleanup, removing thumbnail",
                            image_id
                        );
                        image_deleted = true;
                    } else {
                        error!("Failed to remove unused image {}: {}", image_id, err);
                    }
                }
            }

            if image_deleted {
                let thumbnail_path = self.storage_manager.thumbnail_file_path(image_id);
                if thumbnail_path.exists() {
                    match fs::remove_file(&thumbnail_path) {
                        Ok(_) => {
                            debug!("Removed thumbnail for image {}", image_id);
                            removed_thumbnails += 1;
                        }
                        Err(err) => {
                            error!(
                                "Failed to remove thumbnail for image {}: {}",
                                image_id, err
                            );
                        }
                    }
                }
            }
        }

        let total_removed = removed_images + removed_thumbnails;
        if total_removed > 0 {
            info!(
                "Image cleanup removed {} file(s) ({} images, {} thumbnails)",
                total_removed, removed_images, removed_thumbnails
            );
        } else {
            debug!("Image cleanup found no unused images to remove");
        }

        total_removed
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
