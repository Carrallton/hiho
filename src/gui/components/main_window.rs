use eframe::egui;
use crate::gui::app::{HihoApp, AppState};
use crate::vault::Vault;
use crate::entry::Entry;
use crate::password_generator::generate_secure_password;
use std::sync::{Arc, Mutex};

pub struct MainWindow {
    vault: Arc<Mutex<Vault>>,
    entries: Vec<Entry>,
    selected_entry: Option<usize>,
    new_entry_name: String,
    new_entry_username: String,
    new_entry_password: String,
    search_query: String,
    show_password_generator: bool,
    generated_password: String,
    password_length: usize,
}

impl MainWindow {
    pub fn new(vault: Arc<Mutex<Vault>>) -> Self {
        let entries = {
            let v = vault.lock().unwrap();
            v.list_entries().clone()
        };
        
        Self {
            vault,
            entries,
            selected_entry: None,
            new_entry_name: String::new(),
            new_entry_username: String::new(),
            new_entry_password: String::new(),
            search_query: String::new(),
            show_password_generator: false,
            generated_password: String::new(),
            password_length: 16,
        }
    }

    pub fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, app: &mut HihoApp) {
        ui.horizontal(|ui| {
            if ui.button("🚪 Выйти").clicked() {
                app.state = AppState::Login;
                app.master_password = String::new();
                app.vault = None;
                return;
            }
            
            if ui.button("🔒 Заблокировать").clicked() {
                app.state = AppState::Locked;
                app.vault = None;
                return;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.text_edit_singleline(&mut self.search_query).hint_text("🔍 Поиск...");
            });
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            // Левая панель - список записей
            egui::SidePanel::left("entries_panel")
                .resizable(true)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    self.show_entries_list(ui);
                });
            
            // Правая панель - детали записи
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.show_entry_details(ui, app);
            });
        });
    }

    fn show_entries_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Записи");
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                    let is_selected = self.selected_entry == Some(i);
                    if ui.selectable_label(is_selected, &entry.name).clicked() {
                        self.selected_entry = Some(i);
                    }
                }
            }
        });
        
        ui.separator();
        if ui.button("➕ Добавить запись").clicked() {
            self.selected_entry = None;
            self.new_entry_name.clear();
            self.new_entry_username.clear();
            self.new_entry_password.clear();
        }
    }

    fn show_entry_details(&mut self, ui: &mut egui::Ui, app: &mut HihoApp) {
        if let Some(index) = self.selected_entry {
            if index < self.entries.len() {
                let entry = &self.entries[index];
                ui.heading(&entry.name);
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("👤 Пользователь:");
                    ui.text_edit_singleline(&mut entry.username.clone());
                });
                
                ui.horizontal(|ui| {
                    ui.label("🔑 Пароль:");
                    ui.text_edit_singleline(&mut entry.password.clone()).password(true);
                    if ui.button("📋").clicked() {
                        self.copy_to_clipboard(&entry.password, app);
                    }
                });
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("✏️ Редактировать").clicked() {
                        // Логика редактирования
                    }
                    if ui.button("🗑️ Удалить").clicked() {
                        self.delete_entry(index, app);
                    }
                });
            }
        } else {
            self.show_new_entry_form(ui, app);
        }
    }

    fn show_new_entry_form(&mut self, ui: &mut egui::Ui, app: &mut HihoApp) {
        ui.heading("➕ Новая запись");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("🌐 Название:");
            ui.text_edit_singleline(&mut self.new_entry_name);
        });
        
        ui.horizontal(|ui| {
            ui.label("👤 Пользователь:");
            ui.text_edit_singleline(&mut self.new_entry_username);
        });
        
        ui.horizontal(|ui| {
            ui.label("🔑 Пароль:");
            ui.text_edit_singleline(&mut self.new_entry_password).password(true);
            if ui.button("🎲").clicked() {
                self.show_password_generator = !self.show_password_generator;
            }
        });
        
        if self.show_password_generator {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Длина:");
                ui.add(egui::Slider::new(&mut self.password_length, 8..=128));
                if ui.button("Сгенерировать").clicked() {
                    self.generated_password = generate_secure_password(self.password_length);
                    self.new_entry_password = self.generated_password.clone();
                }
            });
            if !self.generated_password.is_empty() {
                ui.label(format!("Сгенерирован: {}", self.generated_password));
            }
        }
        
        ui.add_space(20.0);
        if ui.button("💾 Сохранить").clicked() {
            self.save_new_entry(app);
        }
    }

    fn copy_to_clipboard(&self, text: &str, app: &mut HihoApp) {
        match ui::clipboard::Clipboard::set_text(text.to_string()) {
            Ok(_) => {
                app.error_message = Some("✅ Скопировано в буфер обмена".to_string());
            }
            Err(e) => {
                app.error_message = Some(format!("❌ Ошибка копирования: {}", e));
            }
        }
    }

    fn delete_entry(&mut self, index: usize, app: &mut HihoApp) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.selected_entry = None;
            self.save_vault(app);
        }
    }

    fn save_new_entry(&mut self, app: &mut HihoApp) {
        if self.new_entry_name.is_empty() {
            app.error_message = Some("Введите название записи".to_string());
            return;
        }
        
        let entry = Entry {
            name: self.new_entry_name.clone(),
            username: self.new_entry_username.clone(),
            password: self.new_entry_password.clone(),
        };
        
        self.entries.push(entry);
        self.save_vault(app);
        
        // Очищаем форму
        self.new_entry_name.clear();
        self.new_entry_username.clear();
        self.new_entry_password.clear();
        self.generated_password.clear();
    }

    fn save_vault(&self, app: &mut HihoApp) {
        if let Some(vault) = &app.vault {
            match vault.lock() {
                Ok(mut v) => {
                    // Обновляем записи в хранилище
                    // Здесь нужно реализовать логику сохранения
                    match v.save_to_file(std::path::Path::new("data\\vault.enc")) {
                        Ok(_) => {
                            app.error_message = Some("✅ Сохранено".to_string());
                        }
                        Err(e) => {
                            app.error_message = Some(format!("❌ Ошибка сохранения: {}", e));
                        }
                    }
                }
                Err(e) => {
                    app.error_message = Some(format!("❌ Ошибка доступа к хранилищу: {}", e));
                }
            }
        }
    }
}