// Module for handling privilege-related functionality

use log::info;
use users::{get_user_by_name, get_current_uid};
use users::switch::{set_both_uid, set_both_gid};
use std::ptr;
use std::io;
use std::io::{Error, ErrorKind};

/// Check if the program has root privileges
pub fn check_root_privileges() -> Result<(), String> {
    if get_current_uid() != 0 {
        return Err("This program must be run as root (sudo) to access the GPIO pins".to_string());
    }
    info!("Running with root privileges");
    Ok(())
}

/// Helper function to clear all supplementary groups
/// Returns Result with () for success or io::Error
fn clear_supplementary_groups() -> io::Result<()> {
    let result = unsafe {
        libc::setgroups(0, ptr::null())
    };
    
    if result != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Drop root privileges to the daemon user
/// 
/// This function checks if we're still running as root first.
/// If privileges have already been dropped, it simply logs and returns success.
pub fn drop_privileges() -> Result<(), Error> {
    // Check if we're still running as root
    let current_uid = get_current_uid();
    if current_uid != 0 {
        info!("Privileges already dropped by led driver (current uid={})", current_uid);
        return Ok(());
    }
    
    // Find the daemon user
    let user = match get_user_by_name("daemon").or_else(|| get_user_by_name("nobody")) {
        Some(user) => user,
        None => {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Could not find daemon or nobody user for privilege dropping"
            ));
        }
    };
    
    let username = user.name().to_string_lossy();
    let uid = user.uid();
    let gid = user.primary_group_id();
    
    info!("Dropping privileges to user {} (uid={}, gid={}) after hardware initialization...", username, uid, gid);
    
    // Clear all supplementary groups
    if let Err(e) = clear_supplementary_groups() {
        return Err(Error::new(ErrorKind::PermissionDenied, 
            format!("Failed to clear supplementary groups: {}", e)));
    }
    
    // Set GID first (required order)
    if let Err(e) = set_both_gid(gid, gid) {
        return Err(Error::new(ErrorKind::PermissionDenied, 
            format!("Failed to set GID: {}", e)));
    }
    
    // Then set UID
    if let Err(e) = set_both_uid(uid, uid) {
        return Err(Error::new(ErrorKind::PermissionDenied, 
            format!("Failed to set UID: {}", e)));
    }
    
    // Verify privileges were dropped
    if get_current_uid() == 0 {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Failed to drop privileges - still running as root!"
        ));
    }
    
    info!("Successfully dropped privileges to user {}", username);
    Ok(())
} 