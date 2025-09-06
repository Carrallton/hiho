use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

const ACTIVITY_FILE: &str = "data\\activity.log";
const CONFIG_FILE: &str = "data\\auto_lock_config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoLockConfig {
    pub timeout_minutes: Option<u64>, // None = отключено
}

impl Default for AutoLockConfig {
    fn default() -> Self {
        Self {
            timeout_minutes: Some(10), // 10 минут по умолчанию
        }
    }
}

pub struct AutoLockManager;

impl AutoLockManager {
    pub fn get_config() -> Result<AutoLockConfig, Box<dyn std::error::Error>> {
        let config_path = Path::new(CONFIG_FILE);
        if config_path.exists() {
            let data = fs::read_to_string(config_path)?;
            let config: AutoLockConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            let config = AutoLockConfig::default();
            Self::save_config(&config)?;
            Ok(config)
        }
    }

    pub fn save_config(config: &AutoLockConfig) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Path::new(CONFIG_FILE);
        std::fs::create_dir_all("data")?;
        let json_data = serde_json::to_string_pretty(config)?;
        fs::write(config_path, json_data)?;
        Ok(())
    }

    pub fn set_timeout(minutes: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = Self::get_config()?;
        config.timeout_minutes = minutes;
        Self::save_config(&config)?;
        println!("✅ Настройки автоблокировки обновлены");
        Ok(())
    }

    pub fn update_activity() -> Result<(), Box<dyn std::error::Error>> {
        let config = Self::get_config()?;
        
        // Если автоблокировка включена
        if config.timeout_minutes.is_some() {
            let activity_path = Path::new(ACTIVITY_FILE);
            fs::write(activity_path, "")?;
        }
        
        Ok(())
    }

    pub fn should_lock() -> Result<bool, Box<dyn std::error::Error>> {
        let config = Self::get_config()?;
        
        // Если автоблокировка отключена
        let timeout_minutes = match config.timeout_minutes {
            Some(minutes) => minutes,
            None => return Ok(false),
        };

        // Проверяем время последней активности
        let activity_path = Path::new(ACTIVITY_FILE);
        if activity_path.exists() {
            let metadata = fs::metadata(activity_path)?;
            let modified = metadata.modified()?;
            let now = SystemTime::now();
            let elapsed = now.duration_since(modified)?.as_secs();
            let timeout_seconds = timeout_minutes * 60;
            
            if elapsed > timeout_seconds {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    pub fn is_locked() -> bool {
        crate::session::SessionManager::is_locked()
    }

    pub fn lock_session() -> Result<(), Box<dyn std::error::Error>> {
        // Создаем файл блокировки сессии
        fs::write("data\\session.lock", "")?;
        // Удаляем файл активности
        let activity_path = Path::new(ACTIVITY_FILE);
        if activity_path.exists() {
            fs::remove_file(activity_path)?;
        }
        Ok(())
    }

    pub fn unlock_session() -> Result<(), Box<dyn std::error::Error>> {
        crate::session::SessionManager::unlock_session()
            .map_err(|e| format!("Ошибка разблокировки: {}", e).into())
    }

    // Добавляем метод unlock
    pub fn unlock() -> Result<(), Box<dyn std::error::Error>> {
        crate::session::SessionManager::unlock_session()
            .map_err(|e| format!("Ошибка разблокировки: {}", e).into())
    }
}