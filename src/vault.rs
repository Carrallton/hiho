use std::fs;
use std::path::Path;
use crate::entry::Entry;
use crate::crypto::{encrypt, decrypt, derive_key};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct VaultData {
    entries: Vec<Entry>,
}

pub struct Vault {
    entries: Vec<Entry>,
    master_password: String,
}

impl Vault {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Vault::new: creating new vault");
        Ok(Vault {
            entries: Vec::new(),
            master_password: password.to_string(),
        })
    }

pub fn load_from_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Vault::load_from_file: path = {:?}", path);
    if !path.exists() {
        println!("Vault::load_from_file: file does not exist");
        return Ok(());
    }

    println!("Vault::load_from_file: reading file");
    let data = fs::read(path)?;
    println!("Vault::load_from_file: file read, size = {}", data.len());
    
    let salt = "hiho_salt_2024";
    println!("Vault::load_from_file: deriving key");
    let key = derive_key(&self.master_password, salt)?;
    println!("Vault::load_from_file: key derived");
    
    let encrypted: crate::crypto::EncryptedData = bincode::deserialize(&data)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    println!("Vault::load_from_file: data deserialized");
    
    let plaintext = decrypt(&encrypted, &key)?;
    println!("Vault::load_from_file: data decrypted, size = {}", plaintext.len());
    
    let vault_data: VaultData = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("JSON parsing error: {}", e))?;
    println!("Vault::load_from_file: JSON parsed, entries count = {}", vault_data.entries.len());
    
    self.entries = vault_data.entries;
    Ok(())
}

    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Vault::save_to_file: path = {:?}", path);
        let vault_data = VaultData {
            entries: self.entries.clone(),
        };
        println!("Vault::save_to_file: vault data prepared, entries count = {}", vault_data.entries.len());
        
        let json_data = serde_json::to_vec(&vault_data)?;
        println!("Vault::save_to_file: JSON serialized, size = {}", json_data.len());
        
        let salt = "hiho_salt_2024";
        println!("Vault::save_to_file: deriving key");
        let key = derive_key(&self.master_password, salt)?;
        println!("Vault::save_to_file: key derived");
        
        let encrypted = encrypt(&json_data, &key)?;
        println!("Vault::save_to_file: data encrypted");
        
        let serialized = bincode::serialize(&encrypted)
            .map_err(|e| format!("Serialization error: {}", e))?;
        println!("Vault::save_to_file: data serialized, size = {}", serialized.len());
        
        fs::write(path, serialized)?;
        println!("Vault::save_to_file: file written");
        Ok(())
    }

    pub fn add_entry(&mut self, entry: Entry) {
        println!("Vault::add_entry: adding entry '{}'", entry.name);
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, index: usize) -> Option<Entry> {
        if index < self.entries.len() {
            let entry = self.entries.remove(index);
            println!("Vault::remove_entry: removed entry '{}'", entry.name);
            Some(entry)
        } else {
            None
        }
    }

    pub fn edit_entry(&mut self, index: usize, username: Option<String>, password: Option<String>) -> Result<(), &'static str> {
        if index >= self.entries.len() {
            return Err("Запись не найдена");
        }
        
        if let Some(new_username) = username {
            println!("Vault::edit_entry: updating username for entry '{}'", self.entries[index].name);
            self.entries[index].username = new_username;
        }
        
        if let Some(new_password) = password {
            println!("Vault::edit_entry: updating password for entry '{}'", self.entries[index].name);
            self.entries[index].password = new_password;
        }
        
        Ok(())
    }

    pub fn get_entries(&self) -> &Vec<Entry> {
        &self.entries
    }
}