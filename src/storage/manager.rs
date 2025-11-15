use log::{debug, error, info, warn};
use std::fs::{self, File, Permissions};
use std::io::{Read, Result as IoResult, Write};
use std::os::unix::fs::chown;
use std::os::unix::fs::PermissionsExt; // For Unix-style permissions
use std::path::{Path, PathBuf};
use uzers::{get_current_uid, get_user_by_name}; // For chown support

// System-wide storage location
pub const DEFAULT_DIR: &str = "/var/lib/led-matrix-controller";

// Path constants for all stored files
pub mod paths {
    // Main data files
    pub const PLAYLIST_FILE: &str = "playlist.json";
    pub const BRIGHTNESS_FILE: &str = "brightness.json";
    pub const IMAGES_DIR: &str = "images";
}

pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    /// Static method to initialize the storage directory with root privileges
    /// Should be called early in program startup, before privilege dropping
    pub fn init_app_directory() -> Result<(), std::io::Error> {
        info!("Initializing storage directory: {}", DEFAULT_DIR);

        // Create the directory if it doesn't exist
        if !Path::new(DEFAULT_DIR).exists() {
            fs::create_dir_all(DEFAULT_DIR)?;
        }

        // Set directory permissions to 700 (rwx------) for owner-only access
        fs::set_permissions(DEFAULT_DIR, Permissions::from_mode(0o700))?;
        debug!("Set permissions on storage directory: 700 (owner access only)");

        // Find daemon user ID, or fall back to nobody if daemon doesn't exist
        let user = get_user_by_name("daemon").or_else(|| {
            debug!("daemon user not found, looking for nobody user");
            get_user_by_name("nobody")
        });

        // Set ownership of the directory if we found a suitable user
        if let Some(user) = user {
            let username = user.name().to_string_lossy();
            let uid = user.uid();
            let gid = user.primary_group_id();

            debug!("Found user {} (uid={}, gid={})", username, uid, gid);

            match chown(DEFAULT_DIR, Some(uid), Some(gid)) {
                Ok(_) => {
                    debug!("Set ownership of storage directory to user {}", username);
                }
                Err(err) => {
                    warn!("Failed to set config directory ownership: {} - this might cause permission issues", err);
                }
            }
        } else {
            warn!("Could not find either daemon or nobody user, leaving config directory owned by root");
        }

        Ok(())
    }

    /// Create a new StorageManager instance
    /// This will handle initial directory setup if run with root privileges
    pub fn new(custom_dir: Option<String>) -> Self {
        // If a custom directory is provided, use it
        let base_dir = if let Some(dir) = custom_dir {
            PathBuf::from(dir)
        } else {
            // Otherwise, use system-wide directory
            let storage_dir = PathBuf::from(DEFAULT_DIR);
            storage_dir
        };

        // Create an instance
        let manager = Self { base_dir };

        // If we have root privileges, properly set up the directory with correct ownership
        if get_current_uid() == 0 {
            if let Err(e) = Self::init_app_directory() {
                error!("Failed to initialize storage directory with root: {}", e);
            }
        } else {
            debug!("Running with reduced privileges, ensuring directory exists");
            // Otherwise just try to create the directory if it doesn't exist
            if let Err(e) = manager.ensure_directory_exists() {
                error!(
                    "Failed to create storage directory with reduced privileges: {}",
                    e
                );
            }
        }

        manager
    }

    // Get the full path for a specific file
    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    // Ensure the base directory exists
    pub fn ensure_directory_exists(&self) -> IoResult<()> {
        if !self.base_dir.exists() {
            debug!(
                "Storage directory doesn't exist, creating with reduced privileges: {:?}",
                self.base_dir
            );
            fs::create_dir_all(&self.base_dir)?;

            // Retain permission setting here as a fallback, but the root init
            // should have already set proper permissions
            #[cfg(unix)]
            {
                let permissions = Permissions::from_mode(0o755); // rwxr-xr-x
                fs::set_permissions(&self.base_dir, permissions)?;
                debug!(
                    "Created storage directory with permissions 755 (fallback): {:?}",
                    self.base_dir
                );
            }
        }
        Ok(())
    }

    fn images_dir(&self) -> PathBuf {
        self.base_dir.join(paths::IMAGES_DIR)
    }

    pub fn ensure_images_dir(&self) -> IoResult<()> {
        let images_dir = self.images_dir();
        if !images_dir.exists() {
            debug!("Images directory doesn't exist, creating: {:?}", images_dir);
            fs::create_dir_all(&images_dir)?;
            #[cfg(unix)]
            {
                let permissions = Permissions::from_mode(0o755);
                fs::set_permissions(&images_dir, permissions)?;
            }
        }
        Ok(())
    }

    pub fn save_image_file(&self, image_id: &str, data: &[u8]) -> IoResult<PathBuf> {
        self.ensure_images_dir()?;
        let path = self.images_dir().join(format!("{}.png", image_id));
        debug!("Writing image file: {:?}", path);
        fs::write(&path, data)?;
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o644);
            fs::set_permissions(&path, permissions)?;
        }
        Ok(path)
    }

    pub fn read_image_file(&self, image_id: &str) -> IoResult<Vec<u8>> {
        let path = self.images_dir().join(format!("{}.png", image_id));
        debug!("Reading image file: {:?}", path);
        fs::read(path)
    }

    pub fn image_file_path(&self, image_id: &str) -> PathBuf {
        self.images_dir().join(format!("{}.png", image_id))
    }

    // Read a file from storage
    pub fn read_file(&self, filename: &str) -> IoResult<String> {
        let file_path = self.get_file_path(filename);
        debug!("Reading file: {:?}", file_path);
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    // Write a file to storage with appropriate permissions
    pub fn write_file(&self, filename: &str, contents: &str) -> IoResult<()> {
        // First ensure directory exists
        self.ensure_directory_exists()?;

        let file_path = self.get_file_path(filename);
        debug!("Writing to file: {:?}", file_path);
        let mut file = File::create(&file_path)?;
        file.write_all(contents.as_bytes())?;

        // Set sensible file permissions (now that we've dropped privileges)
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o644); // rw-r--r--
            fs::set_permissions(&file_path, permissions)?;
        }

        debug!(
            "Successfully wrote {} bytes to {}",
            contents.len(),
            filename
        );
        Ok(())
    }

    // Check if a file exists
    pub fn file_exists(&self, filename: &str) -> bool {
        let exists = self.get_file_path(filename).exists();
        debug!("Checking if file '{}' exists: {}", filename, exists);
        exists
    }

    // Note: kept delete_file since it could be useful later,
    // but marked with #[allow(dead_code)] to suppress warnings
    #[allow(dead_code)]
    pub fn delete_file(&self, filename: &str) -> IoResult<()> {
        let file_path = self.get_file_path(filename);
        if file_path.exists() {
            debug!("Deleting file: {:?}", file_path);
            fs::remove_file(file_path)?;
            info!("Deleted file: {}", filename);
        } else {
            debug!("File to delete doesn't exist: {}", filename);
        }
        Ok(())
    }
}
