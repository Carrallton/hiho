#![cfg_attr(not(feature = "cli"), allow(dead_code, unused_imports))]

mod crypto;
mod vault;
mod entry;
#[cfg(feature = "cli")]
mod cli;
mod password_generator;
mod session;

use cli::{Cli, Commands};
use entry::Entry;
use vault::Vault;
use std::path::Path;
use std::error::Error;
use clap::Parser;
use password_generator::{generate_password, generate_secure_password};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::fs::File;
use std::io::{BufReader, BufRead}; // Убрал Write
use session::SessionManager;

const VAULT_FILE: &str = "data\\vault.enc";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    // Проверяем автоблокировку перед выполнением команд
    if !matches!(cli.command, Commands::Unlock | Commands::Init) {
        if SessionManager::is_locked() {
            println!("🔒 Сессия заблокирована. Используйте 'hiho unlock' для разблокировки.");
            return Ok(());
        }
    }
    
    match &cli.command {
        Commands::Init => {
            let vault_path = Path::new(VAULT_FILE);
            
            // Проверяем, существует ли уже хранилище
            if vault_path.exists() {
                println!("❌ Хранилище уже существует!");
                println!("Используйте существующее хранилище или удалите файл {} для создания нового", VAULT_FILE);
                return Ok(());
            }
            
            println!("🔐 Инициализация хранилища hiho...");
            let password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let vault = Vault::new(&password)?;
            
            std::fs::create_dir_all("data")?;
            vault.save_to_file(vault_path)?;
            println!("✅ Хранилище создано!");
        }
        Commands::Add { name, username, password, length } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
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
            
            let entries = vault.get_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            println!("📋 Ваши записи:");
            for (i, entry) in entries.iter().enumerate() {
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
            
            let entries = vault.get_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            if let Some(entry) = find_entry(&vault, name_or_index)? {
                copy_to_clipboard(&entry.password)?;
                println!("✅ Пароль для '{}' скопирован в буфер обмена!", entry.name);
            } else {
                println!("❌ Запись '{}' не найдена!", name_or_index);
            }
        }
        
        Commands::Remove { name_or_index } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.get_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            if let Some((index, entry)) = find_entry_with_index(&vault, name_or_index)? {
                println!("🗑️  Удалить запись: {} - {}?", entry.name, entry.username);
                println!("Введите 'y' для подтверждения:");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() == "y" {
                    vault.remove_entry(index);
                    vault.save_to_file(Path::new(VAULT_FILE))?;
                    println!("✅ Запись удалена!");
                } else {
                    println!("❌ Удаление отменено");
                }
            } else {
                println!("❌ Запись '{}' не найдена!", name_or_index);
            }
        }
        
        Commands::Search { query } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.get_entries();
            let results: Vec<(usize, &Entry)> = entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.name.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();
                
            if results.is_empty() {
                println!("🔍 Ничего не найдено по запросу '{}'", query);
            } else {
                println!("🔍 Найдено {} записей:", results.len());
                for (i, (index, entry)) in results.iter().enumerate() {
                    println!("{}. {}: {} - {}", i+1, entry.name, entry.username, entry.password);
                }
            }
        }
        
        Commands::Edit { name_or_index, username, password, length } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.get_entries();
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
                println!("✏️  Редактирование записи: {} - {}", entry.name, entry.username);
                
                let new_username = username.clone().unwrap_or_else(|| entry.username.clone());
                let new_password = match password {
                    Some(p) => p.clone(),
                    None => {
                        if username.is_some() || password.is_some() {
                            // Если указаны параметры, но не пароль, генерируем
                            println!("Генерируем новый пароль длиной {} символов...", length);
                            generate_secure_password(*length)
                        } else {
                            entry.password.clone()
                        }
                    }
                };
                
                match vault.edit_entry(index, Some(new_username), Some(new_password)) {
                    Ok(_) => {
                        vault.save_to_file(Path::new(VAULT_FILE))?;
                        println!("✅ Запись обновлена!");
                    }
                    Err(e) => {
                        println!("❌ Ошибка редактирования: {}", e);
                    }
                }
            } else {
                println!("❌ Запись '{}' не найдена!", name_or_index);
            }
        }
        
        Commands::Export { file, format } => {
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            let entries = vault.get_entries();
            if entries.is_empty() {
                println!("📭 Хранилище пусто!");
                return Ok(());
            }
            
            match format.as_str() {
                "json" => {
                    let json_data = serde_json::to_string_pretty(entries)?;
                    std::fs::write(file, json_data)?;
                    println!("✅ Данные экспортированы в {} ({} записей)", file, entries.len());
                }
                "csv" => {
                    let mut csv_data = String::new();
                    csv_data.push_str("name,username,password\n");
                    for entry in entries {
                        csv_data.push_str(&format!("{},{},{}\n", 
                            escape_csv(&entry.name), 
                            escape_csv(&entry.username), 
                            escape_csv(&entry.password)
                        ));
                    }
                    std::fs::write(file, csv_data)?;
                    println!("✅ Данные экспортированы в {} ({} записей)", file, entries.len());
                }
                _ => {
                    println!("❌ Неподдерживаемый формат: {}", format);
                }
            }
        }
        
        Commands::Import { file, format } => {
            if !Path::new(file).exists() {
                println!("❌ Файл {} не найден!", file);
                return Ok(());
            }
            
            let master_password = rpassword::prompt_password("Введите мастер-пароль: ")?;
            let mut vault = Vault::new(&master_password)?;
            
            // Загружаем существующие данные если есть
            if Path::new(VAULT_FILE).exists() {
                vault.load_from_file(Path::new(VAULT_FILE))?;
            }
            
            match format.as_str() {
                "json" => {
                    let file_content = std::fs::read_to_string(file)?;
                    let imported_entries: Vec<Entry> = serde_json::from_str(&file_content)?;
                    let count = imported_entries.len();
                    
                    for entry in imported_entries {
                        vault.add_entry(entry);
                    }
                    
                    vault.save_to_file(Path::new(VAULT_FILE))?;
                    println!("✅ Импортировано {} записей из {}", count, file);
                }
                "csv" => {
                    let file_handle = File::open(file)?;
                    let reader = BufReader::new(file_handle);
                    let mut entries_count = 0;
                    
                    for (line_num, line) in reader.lines().enumerate() {
                        let line = line?;
                        if line_num == 0 {
                            // Пропускаем заголовок
                            continue;
                        }
                        
                        let fields: Vec<&str> = line.split(',').collect();
                        if fields.len() >= 3 {
                            let entry = Entry {
                                name: unescape_csv(fields[0]),
                                username: unescape_csv(fields[1]),
                                password: unescape_csv(fields[2]),
                            };
                            vault.add_entry(entry);
                            entries_count += 1;
                        }
                    }
                    
                    vault.save_to_file(Path::new(VAULT_FILE))?;
                    println!("✅ Импортировано {} записей из {}", entries_count, file);
                }
                _ => {
                    println!("❌ Неподдерживаемый формат: {}", format);
                }
            }
        }
        
        Commands::LockConfig { timeout, show } => {
            if *show {
                println!("⏰ Автоблокировка: отключена (временно)");
            } else if let Some(minutes) = timeout {
                println!("✅ Автоблокировка установлена на {} минут (временно)", minutes);
            } else {
                println!("❌ Укажите --timeout или --show");
            }
        }
        
        Commands::Unlock => {
            if SessionManager::is_locked() {
                let password = rpassword::prompt_password("Введите мастер-пароль для разблокировки: ")?;
                let _vault = Vault::new(&password)?;
                
                // Простая проверка правильности пароля
                if Path::new(VAULT_FILE).exists() {
                    let mut test_vault = Vault::new(&password)?;
                    match test_vault.load_from_file(Path::new(VAULT_FILE)) {
                        Ok(_) => {
                            SessionManager::unlock_session()?;
                            println!("✅ Сессия разблокирована!");
                        }
                        Err(_) => {
                            println!("❌ Неверный пароль");
                        }
                    }
                } else {
                    SessionManager::unlock_session()?;
                    println!("✅ Сессия разблокирована!");
                }
            } else {
                println!("🔓 Сессия не заблокирована");
            }
        }
    }
    
    Ok(())
}

// Вспомогательная функция для поиска записи
fn find_entry<'a>(vault: &'a Vault, name_or_index: &str) -> Result<Option<&'a Entry>, Box<dyn Error>> {
    let entries = vault.get_entries();
    
    // Пытаемся найти по индексу
    if let Ok(index) = name_or_index.parse::<usize>() {
        if index > 0 && index <= entries.len() {
            return Ok(Some(&entries[index - 1]));
        }
    }
    
    // Ищем по имени
    for entry in entries {
        if entry.name == *name_or_index {
            return Ok(Some(entry));
        }
    }
    
    Ok(None)
}

// Вспомогательная функция для поиска записи с индексом
fn find_entry_with_index<'a>(vault: &'a Vault, name_or_index: &str) -> Result<Option<(usize, &'a Entry)>, Box<dyn Error>> {
    let entries = vault.get_entries();
    
    // Пытаемся найти по индексу
    if let Ok(index) = name_or_index.parse::<usize>() {
        if index > 0 && index <= entries.len() {
            return Ok(Some((index - 1, &entries[index - 1])));
        }
    }
    
    // Ищем по имени
    for (i, entry) in entries.iter().enumerate() {
        if entry.name == *name_or_index {
            return Ok(Some((i, entry)));
        }
    }
    
    Ok(None)
}

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(text.to_owned())?;
    Ok(())
}

// Вспомогательные функции для CSV
fn escape_csv(text: &str) -> String {
    if text.contains(',') || text.contains('"') || text.contains('\n') {
        format!("\"{}\"", text.replace("\"", "\"\""))
    } else {
        text.to_string()
    }
}

fn unescape_csv(text: &str) -> String {
    if text.starts_with('"') && text.ends_with('"') {
        text[1..text.len()-1].replace("\"\"", "\"")
    } else {
        text.to_string()
    }
}