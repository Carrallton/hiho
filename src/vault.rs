use std::fs;
use std::path::Path;
use crate::crypto::{encrypt, decrypt, EncryptedData, derive_key};
use crate::entry::Entry;
use std::error::Error;

pub struct Vault {
    entries: Vec<Entry>,
    key: [u8; 32],
}

impl Vault {
    pub fn new(master_password: &str) -> Result<Self, Box<dyn Error>> {
        let salt = "hiho_salt_2024";
        let key = derive_key(master_password, salt)?;
        
        Ok(Vault {
            entries: Vec::new(),
            key,
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
    
    // Поиск записей по части имени (новый метод)
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
    
    // Редактирование записи (новый метод)
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
}