use std::fs::{self, File, Permissions};
use std::io::{Read, Write, Result as IoResult};
use std::path::PathBuf;
use log::{info, error, debug, warn};
use std::os::unix::fs::PermissionsExt; // For Unix-style permissions

// Default storage location - use a system-wide location in /var/lib
const DEFAULT_DIR: &str = "/var/lib/led-matrix-controller";

pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    pub fn new(custom_dir: Option<String>) -> Self {
        // If a custom directory is provided, use it
        let base_dir = if let Some(dir) = custom_dir {
            debug!("Using custom storage directory: {}", dir);
            PathBuf::from(dir)
        } else {
            // Otherwise, use system-wide directory
            let storage_dir = PathBuf::from(DEFAULT_DIR);
            debug!("Using system-wide storage directory: {:?}", storage_dir);
            storage_dir
        };
        
        // Create the instance
        let manager = Self { base_dir };
        
        // Ensure the directory exists with appropriate permissions
        if let Err(e) = manager.ensure_directory_exists() {
            error!("Failed to create storage directory: {}", e);
        }
        
        manager
    }
    
    // Get the full path for a specific file
    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }
    
    // Ensure the base directory exists with world-writable permissions
    pub fn ensure_directory_exists(&self) -> IoResult<()> {
        if !self.base_dir.exists() {
            debug!("Creating storage directory: {:?}", self.base_dir);
            fs::create_dir_all(&self.base_dir)?;
            
            // Set directory permissions to 777 (rwxrwxrwx) so anyone can read/write
            #[cfg(unix)]
            {
                let permissions = Permissions::from_mode(0o777);
                fs::set_permissions(&self.base_dir, permissions)?;
                info!("Created world-writable storage directory: {:?}", self.base_dir);
                warn!("Using world-writable directory - this is a temporary solution until privilege dropping is implemented");
            }
            
            #[cfg(not(unix))]
            {
                info!("Created storage directory: {:?}", self.base_dir);
                warn!("Permission changes not supported on this platform");
            }
        }
        Ok(())
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
    
    // Write a file to storage with world-writable permissions
    pub fn write_file(&self, filename: &str, contents: &str) -> IoResult<()> {
        // First ensure directory exists
        self.ensure_directory_exists()?;
        
        let file_path = self.get_file_path(filename);
        debug!("Writing to file: {:?}", file_path);
        let mut file = File::create(&file_path)?;
        file.write_all(contents.as_bytes())?;
        
        // Set file permissions to 666 (rw-rw-rw-) so anyone can read/write
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o666);
            fs::set_permissions(&file_path, permissions)?;
        }
        
        debug!("Successfully wrote {} bytes to {}", contents.len(), filename);
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