use std::fs;
use std::path::Path;

const SESSION_FILE: &str = "data\\session.lock";

pub struct SessionManager;

impl SessionManager {
    pub fn lock_session() -> std::io::Result<()> {
        fs::write(SESSION_FILE, "")?;
        Ok(())
    }

    pub fn unlock_session() -> std::io::Result<()> {
        if Path::new(SESSION_FILE).exists() {
            fs::remove_file(SESSION_FILE)?;
        }
        Ok(())
    }

    pub fn is_locked() -> bool {
        Path::new(SESSION_FILE).exists()
    }
}