use std::fs;
use std::path::Path;
use crate::crypto::{encrypt, decrypt, EncryptedData, derive_key};
use crate::entry::Entry;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultConfig {
    pub auto_lock_timeout: Option<u64>, // минуты, None = отключено
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            auto_lock_timeout: Some(10), // 10 минут по умолчанию
        }
    }
}

pub struct Vault {
    entries: Vec<Entry>,
    key: [u8; 32],
    config: VaultConfig,
}

impl Vault {
    pub fn new(master_password: &str) -> Result<Self, Box<dyn Error>> {
        let salt = "hiho_salt_2024";
        let key = derive_key(master_password, salt)?;
        
        // Загружаем конфигурацию если есть
        let config = Self::load_config()?;
        
        Ok(Vault {
            entries: Vec::new(),
            key,
            config,
        })
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        if !path.exists() {
            return Ok(());
        }

        let data = fs::read(path)?;
        let encrypted: EncryptedData = bincode::deserialize(&data)?;
        let plaintext = decrypt(&encrypted, &self.key)?;
        self.entries = serde_json::from_slice(&plaintext)?;
        Ok(())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json_data = serde_json::to_vec(&self.entries)?;
        let encrypted = encrypt(&json_data, &self.key)?;
        let serialized = bincode::serialize(&encrypted)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn list_entries(&self) -> &Vec<Entry> {
        &self.entries
    }
    
    // Удаление записи
    pub fn remove_entry(&mut self, index: usize) -> Option<Entry> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }
    
    // Поиск записи по имени
    pub fn find_entry_by_name(&self, name: &str) -> Option<(usize, &Entry)> {
        self.entries.iter().enumerate()
            .find(|(_, entry)| entry.name == name)
    }
    
    // Поиск записей по части имени
    pub fn search_entries(&self, query: &str) -> Vec<(usize, &Entry)> {
        self.entries.iter().enumerate()
            .filter(|(_, entry)| {
                entry.name.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }
    
    // Получение записи по индексу
    pub fn get_entry_by_index(&self, index: usize) -> Option<&Entry> {
        self.entries.get(index)
    }
    
    // Получение индекса записи по имени
    pub fn get_index_by_name(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.name == name)
    }
    
    // Редактирование записи
    pub fn edit_entry(&mut self, index: usize, username: Option<String>, password: Option<String>) -> Result<(), &'static str> {
        if index >= self.entries.len() {
            return Err("Запись не найдена");
        }
        
        if let Some(new_username) = username {
            self.entries[index].username = new_username;
        }
        
        if let Some(new_password) = password {
            self.entries[index].password = new_password;
        }
        
        Ok(())
    }
    
    // Работа с конфигурацией
    fn config_file_path() -> String {
        "data\\config.json".to_string()
    }

    fn load_config() -> Result<VaultConfig, Box<dyn Error>> {
        let config_path = Self::config_file_path();
        if Path::new(&config_path).exists() {
            let data = fs::read_to_string(&config_path)?;
            let config: VaultConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            Ok(VaultConfig::default())
        }
    }

    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::config_file_path();
        let json_data = serde_json::to_string_pretty(&self.config)?;
        fs::write(config_path, json_data)?;
        Ok(())
    }

    pub fn get_config(&self) -> &VaultConfig {
        &self.config
    }

    pub fn set_auto_lock_timeout(&mut self, timeout_minutes: Option<u64>) -> Result<(), Box<dyn Error>> {
        self.config.auto_lock_timeout = timeout_minutes;
        self.save_config()?;
        Ok(())
    }
}