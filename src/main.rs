mod crypto;
mod vault;
mod entry;
mod cli;
mod password_generator;

use cli::{Cli, Commands};
use entry::Entry;
use vault::Vault;
use std::path::Path;
use std::error::Error;
use clap::Parser;
use password_generator::{generate_password, generate_secure_password};
use clipboard::{ClipboardContext, ClipboardProvider};

const VAULT_FILE: &str = "data\\vault.enc";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Init => {
            println!("🔐 Инициализация хранилища hiho...");
            let password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let vault = Vault::new(&password)?;
            
            // Создаем директорию если её нет
            std::fs::create_dir_all("data")?;
            
            vault.save_to_file(Path::new(VAULT_FILE))?;
            println!("✅ Хранилище создано!");
        }
        Commands::Add { name, username, password, length } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            // Генерируем пароль если не указан
            let final_password = match password {
                Some(p) => p.clone(),
                None => {
                    println!("Генерируем пароль длиной {} символов...", length);
                    generate_secure_password(*length)
                }
            };
            
            let entry = Entry {
                name: name.clone(),
                username: username.clone(),
                password: final_password,
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
        Commands::Generate { length, secure } => {
            let password = if *secure {
                generate_secure_password(*length)
            } else {
                generate_password(*length)
            };
            println!("🔐 Сгенерированный пароль: {}", password);
        }
        Commands::Copy { name_or_index } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.list_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            // Пытаемся найти по индексу
            if let Ok(index) = name_or_index.parse::<usize>() {
                if index > 0 && index <= entries.len() {
                    let entry = &entries[index - 1];
                    copy_to_clipboard(&entry.password)?;
                    println!("✅ Пароль для '{}' скопирован в буфер обмена!", entry.name);
                    return Ok(());
                }
            }
            
            // Ищем по имени
            for entry in entries {
                if entry.name == *name_or_index {
                    copy_to_clipboard(&entry.password)?;
                    println!("✅ Пароль для '{}' скопирован в буфер обмена!", entry.name);
                    return Ok(());
                }
            }
            
            println!("❌ Запись '{}' не найдена!", name_or_index);
        }
        Commands::Remove { name_or_index } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.list_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            let mut found_index = None;
            
            // Пытаемся найти по индексу
            if let Ok(index) = name_or_index.parse::<usize>() {
                if index > 0 && index <= entries.len() {
                    found_index = Some(index - 1);
                }
            } else {
                // Ищем по имени
                for (i, entry) in entries.iter().enumerate() {
                    if entry.name == *name_or_index {
                        found_index = Some(i);
                        break;
                    }
                }
            }
            
            if let Some(index) = found_index {
                let entry = &entries[index];
                println!("🗑️  Удалить запись: {} - {}?", entry.name, entry.username);
                println!("Введите 'y' для подтверждения:");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() == "y" {
                    // Удаляем запись (нужно добавить метод в Vault)
                    println!("❌ Удаление пока не реализовано - функция будет добавлена в следующем шаге!");
                }
            } else {
                println!("❌ Запись '{}' не найдена!", name_or_index);
            }
        }
    }
    
    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(text.to_owned())?;
    Ok(())
}