use uuid::Uuid;

// Helper function to generate UUID strings for default values
pub fn generate_uuid_string() -> String {
    Uuid::new_v4().to_string()
}
