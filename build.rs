fn main() {
    // Tell Cargo to rerun this script if the embedded files change
    println!("cargo:rerun-if-changed=static/");
    
    // Add any other build scripts as needed
} 