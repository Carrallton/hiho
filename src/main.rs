mod crypto;
mod vault;
mod entry;
mod cli;

use cli::{Cli, Commands};
use entry::Entry;
use vault::Vault;
use std::path::Path;
use std::error::Error;
use clap::Parser;

const VAULT_FILE: &str = "data\\vault.enc";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Init => {
            println!("🔐 Инициализация хранилища hiho...");
            let password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let vault = Vault::new(&password)?;  // Убрал mut
            
            // Создаем директорию если её нет
            std::fs::create_dir_all("data")?;
            
            vault.save_to_file(Path::new(VAULT_FILE))?;
            println!("✅ Хранилище создано!");
        }
        Commands::Add { name, username, password } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entry = Entry {
                name: name.clone(),
                username: username.clone(),
                password: password.clone(),
            };
            
            vault.add_entry(entry);
            vault.save_to_file(Path::new(VAULT_FILE))?;
            println!("✅ Запись добавлена!");
        }
        Commands::List => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            println!("📋 Ваши записи:");
            for (i, entry) in vault.list_entries().iter().enumerate() {
                println!("{}. {}: {} - {}", i+1, entry.name, entry.username, entry.password);
            }
        }
    }
    
    Ok(())
}