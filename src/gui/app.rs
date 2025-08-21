use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::Path;

// Импортируем настоящие структуры из нашего крейта
use hiho::{Vault, Entry};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Login,
    Main,
    AddEntry,
    EditEntry(usize),
    PasswordGenerator,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PasswordStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: usize,
    pub use_uppercase: bool,
    pub use_lowercase: bool,
    pub use_numbers: bool,
    pub use_symbols: bool,
    pub exclude_ambiguous: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 16,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
            exclude_ambiguous: false,
        }
    }
}

pub struct HihoApp {
    pub state: AppState,
    pub vault: Option<Arc<Mutex<Vault>>>,
    pub master_password: String,
    pub error_message: Option<String>,
    pub entries: Vec<Entry>,
    pub search_query: String,
    pub selected_entry: Option<usize>,
    
    // Для формы добавления/редактирования
    pub form_name: String,
    pub form_username: String,
    pub form_password: String,
    pub show_password_generator: bool,
    pub generated_password: String,
    
    // Новые поля для генератора паролей
    pub password_options: PasswordOptions,
}

impl Default for HihoApp {
    fn default() -> Self {
        Self {
            state: AppState::Login,
            vault: None,
            master_password: String::new(),
            error_message: None,
            entries: Vec::new(),
            search_query: String::new(),
            selected_entry: None,
            
            form_name: String::new(),
            form_username: String::new(),
            form_password: String::new(),
            show_password_generator: false,
            generated_password: String::new(),
            
            // Новые поля
            password_options: PasswordOptions::default(),
        }
    }
}

impl eframe::App for HihoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Login => {
                    self.show_login_screen(ui);
                }
                AppState::Main => {
                    self.show_main_screen(ui);
                }
                AppState::AddEntry => {
                    self.show_entry_form(ui, "Добавить запись");
                }
                AppState::EditEntry(_) => {
                    self.show_entry_form(ui, "Редактировать запись");
                }
                AppState::PasswordGenerator => {
                    self.show_password_generator_main(ui);
                }
                AppState::Locked => {
                    self.show_locked_screen(ui);
                }
            }
            
            // Отображение ошибок
            if let Some(error) = &self.error_message {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }
}

impl HihoApp {
    fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("🔐 hiho - Менеджер паролей уровня NSA");
            ui.add_space(30.0);
            
            ui.horizontal(|ui| {
                ui.label("🔑 Мастер-пароль:");
                let password_field = ui.add(
                    egui::TextEdit::singleline(&mut self.master_password)
                        .password(true)
                        .hint_text("Введите пароль")
                        .desired_width(200.0)
                );
                
                if password_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.attempt_login();
                }
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("🔓 Войти").clicked() {
                    self.attempt_login();
                }
                
                if ui.button("🆕 Создать хранилище").clicked() {
                    self.create_vault();
                }
            });
        });
    }

    fn show_main_screen(&mut self, ui: &mut egui::Ui) {
        // Верхняя панель
        ui.horizontal(|ui| {
            if ui.button("🚪 Выйти").clicked() {
                self.state = AppState::Login;
                self.master_password = String::new();
                self.vault = None;
                return;
            }
            
            if ui.button("🔒 Заблокировать").clicked() {
                self.state = AppState::Locked;
                return;
            }
            
            if ui.button("🎲 Генератор").clicked() {
                self.state = AppState::PasswordGenerator;
                return;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("🔍 Поиск..."));
            });
        });
        
        ui.separator();
        
        // Основная область с записями
        ui.horizontal(|ui| {
            egui::SidePanel::left("entries_panel")
                .resizable(true)
                .default_width(250.0)
                .show_inside(ui, |ui| {
                    self.show_entries_list(ui);
                });
            
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.show_entry_details(ui);
            });
        });
    }

    fn show_entries_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Записи");
        ui.separator();
        
        // Кнопка добавления
        if ui.button("➕ Добавить запись").clicked() {
            self.prepare_new_entry_form();
            self.state = AppState::AddEntry;
            return;
        }
        
        ui.add_space(10.0);
        
        // Отображаем список записей
        self.show_filtered_entries(ui);
    }

    fn show_filtered_entries(&mut self, ui: &mut egui::Ui) {
        // Создаем копии данных
        let search_query = self.search_query.clone();
        let entries = self.entries.clone();
        let selected_index = self.selected_entry;
        
        // Создаем список индексов кликнутых элементов
        let mut clicked_indices = Vec::new();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Фильтруем записи
            let mut filtered_entries = Vec::new();
            for (index, entry) in entries.into_iter().enumerate() {
                if search_query.is_empty() || 
                   entry.name.to_lowercase().contains(&search_query.to_lowercase()) {
                    filtered_entries.push((index, entry));
                }
            }
            
            if filtered_entries.is_empty() {
                if search_query.is_empty() {
                    ui.label("📭 Нет записей");
                } else {
                    ui.label("🔍 Ничего не найдено");
                }
            } else {
                for (index, entry) in filtered_entries {
                    let is_selected = selected_index == Some(index);
                    if ui.selectable_label(is_selected, &entry.name).clicked() {
                        clicked_indices.push(index);
                    }
                }
            }
        });
        
        // Обрабатываем клики после отображения
        if let Some(&clicked_index) = clicked_indices.first() {
            self.selected_entry = Some(clicked_index);
        }
    }

    fn show_entry_details(&mut self, ui: &mut egui::Ui) {
        // Создаем копию selected_entry чтобы избежать заимствования
        let selected_index = self.selected_entry;
        let entries_len = self.entries.len();
        
        if let Some(index) = selected_index {
            if index < entries_len {
                // Создаем копию записи
                let entry = self.entries[index].clone();
                
                ui.heading(&entry.name);
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("👤 Пользователь:");
                    ui.label(&entry.username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("🔑 Пароль:");
                    ui.label("••••••••");
                    if ui.button("📋").clicked() {
                        self.copy_to_clipboard(&entry.password);
                    }
                });
                
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("✏️ Редактировать").clicked() {
                        self.prepare_edit_entry_form(index);
                        self.state = AppState::EditEntry(index);
                    }
                    if ui.button("🗑️ Удалить").clicked() {
                        self.delete_entry(index);
                        self.selected_entry = None;
                    }
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("📋 Выберите запись из списка");
                ui.add_space(20.0);
                if ui.button("➕ Добавить первую запись").clicked() {
                    self.prepare_new_entry_form();
                    self.state = AppState::AddEntry;
                }
            });
        }
    }

    fn show_entry_form(&mut self, ui: &mut egui::Ui, title: &str) {
        ui.heading(title);
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🌐 Название:");
                ui.text_edit_singleline(&mut self.form_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("👤 Пользователь:");
                ui.text_edit_singleline(&mut self.form_username);
            });
            
            ui.horizontal(|ui| {
                ui.label("🔑 Пароль:");
                ui.add(egui::TextEdit::singleline(&mut self.form_password).password(true));
                if ui.button("🎲").clicked() {
                    self.show_password_generator = true;
                }
            });
            
            // Показываем генератор паролей как popup
            if self.show_password_generator {
                self.show_password_generator_popup(ui);
            }
            
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if ui.button("💾 Сохранить").clicked() {
                    self.save_entry();
                }
                if ui.button("❌ Отмена").clicked() {
                    self.state = AppState::Main;
                    self.selected_entry = None;
                }
            });
        });
    }

    fn show_locked_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("🔒 Сессия заблокирована");
            ui.add_space(30.0);
            
            if ui.button("🔓 Разблокировать").clicked() {
                self.state = AppState::Login;
            }
        });
    }

    fn show_password_generator_main(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("🔙 Назад").clicked() {
                self.state = AppState::Main;
            }
            ui.heading("🎲 Генератор паролей");
        });
        ui.separator();
        
        // Используем тот же popup, но в основном окне
        self.show_password_generator_popup(ui);
    }

    fn show_password_generator_popup(&mut self, ui: &mut egui::Ui) {
        // Создаем копию для избежания конфликта заимствований
        let mut show_popup = self.show_password_generator;
        
        egui::Window::new("🎲 Генератор паролей")
            .resizable(true)
            .default_width(400.0)
            .default_height(500.0)
            .open(&mut show_popup)
            .show(ui.ctx(), |ui| {
                self.show_password_generator_content(ui);
            });
            
        // Обновляем значение после использования
        self.show_password_generator = show_popup;
    }

    fn show_password_generator_content(&mut self, ui: &mut egui::Ui) {
        // Создаем копии данных для избежания конфликта заимствований
        let generated_password = self.generated_password.clone();
        let password_options = self.password_options.clone();
        let form_password = self.form_password.clone();
        
        // Переменные для обновления состояния
        let mut new_length = password_options.length;
        let mut new_use_uppercase = password_options.use_uppercase;
        let mut new_use_lowercase = password_options.use_lowercase;
        let mut new_use_numbers = password_options.use_numbers;
        let mut new_use_symbols = password_options.use_symbols;
        let mut new_exclude_ambiguous = password_options.exclude_ambiguous;
        let mut new_generated_password = generated_password.clone();
        let mut new_form_password = form_password.clone();
        let mut close_generator = false;
        let mut use_password = false;
        
        ui.vertical(|ui| {
            // Настройки длины
            ui.horizontal(|ui| {
                ui.label("Длина пароля:");
                ui.add(egui::Slider::new(&mut new_length, 4..=128));
                ui.label(format!("{}", new_length));
            });
            
            ui.separator();
            
            // Опции символов
            ui.checkbox(&mut new_use_uppercase, "Заглавные буквы (A-Z)");
            ui.checkbox(&mut new_use_lowercase, "Строчные буквы (a-z)");
            ui.checkbox(&mut new_use_numbers, "Цифры (0-9)");
            ui.checkbox(&mut new_use_symbols, "Символы (!@#$%^&*)");
            
            ui.separator();
            
            ui.checkbox(&mut new_exclude_ambiguous, "Исключить похожие символы (0,O,l,1)");
            
            ui.separator();
            
            // Кнопка генерации
            if ui.button("🔄 Сгенерировать").clicked() {
                // Обновляем опции перед генерацией
                self.password_options.length = new_length;
                self.password_options.use_uppercase = new_use_uppercase;
                self.password_options.use_lowercase = new_use_lowercase;
                self.password_options.use_numbers = new_use_numbers;
                self.password_options.use_symbols = new_use_symbols;
                self.password_options.exclude_ambiguous = new_exclude_ambiguous;
                
                new_generated_password = self.generate_advanced_password();
            }
            
            ui.separator();
            
            // Отображение сгенерированного пароля
            if !new_generated_password.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Сгенерированный пароль:");
                    let mut password_copy = new_generated_password.clone();
                    ui.add(egui::TextEdit::singleline(&mut password_copy)
                        .desired_width(200.0));
                    if ui.button("📋").clicked() {
                        self.copy_to_clipboard(&new_generated_password);
                    }
                });
                
                // Индикатор сложности (используем копию)
                let strength = self.calculate_password_strength(&new_generated_password);
                self.show_password_strength_indicator(ui, strength);
                
                ui.separator();
                
                // Кнопки действий
                ui.horizontal(|ui| {
                    if ui.button("✅ Использовать").clicked() {
                        use_password = true;
                        new_form_password = new_generated_password.clone();
                        close_generator = true;
                    }
                    if ui.button("❌ Закрыть").clicked() {
                        close_generator = true;
                        new_generated_password.clear();
                    }
                });
            } else {
                ui.label("Нажмите 'Сгенерировать' для создания пароля");
            }
        });
        
        // Обновляем состояние после закрытия окна
        if close_generator {
            self.show_password_generator = false;
            self.generated_password = new_generated_password;
            if use_password {
                self.form_password = new_form_password;
            }
        } else {
            // Обновляем опции
            self.password_options.length = new_length;
            self.password_options.use_uppercase = new_use_uppercase;
            self.password_options.use_lowercase = new_use_lowercase;
            self.password_options.use_numbers = new_use_numbers;
            self.password_options.use_symbols = new_use_symbols;
            self.password_options.exclude_ambiguous = new_exclude_ambiguous;
            self.generated_password = new_generated_password;
        }
    }

    // Вспомогательные методы
    fn prepare_new_entry_form(&mut self) {
        self.form_name.clear();
        self.form_username.clear();
        self.form_password.clear();
        self.generated_password.clear();
        self.show_password_generator = false;
    }

    fn prepare_edit_entry_form(&mut self, index: usize) {
        if index < self.entries.len() {
            let entry = &self.entries[index];
            self.form_name = entry.name.clone();
            self.form_username = entry.username.clone();
            self.form_password = entry.password.clone();
            self.generated_password.clear();
            self.show_password_generator = false;
        }
    }

    fn generate_advanced_password(&self) -> String {
        use rand::{Rng, thread_rng};
        
        let mut charset = String::new();
        
        if self.password_options.use_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        
        if self.password_options.use_lowercase {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }
        
        if self.password_options.use_numbers {
            charset.push_str("0123456789");
        }
        
        if self.password_options.use_symbols {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }
        
        // Исключаем похожие символы если нужно
        let charset = if self.password_options.exclude_ambiguous {
            charset.replace("0", "").replace("O", "").replace("l", "").replace("I", "")
        } else {
            charset
        };
        
        if charset.is_empty() {
            return String::new();
        }
        
        let mut rng = thread_rng();
        (0..self.password_options.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect()
    }

    fn calculate_password_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;
        
        // Длина
        if password.len() >= 8 { score += 1; }
        if password.len() >= 12 { score += 1; }
        if password.len() >= 16 { score += 1; }
        
        // Разнообразие символов
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_symbol = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if has_upper { score += 1; }
        if has_lower { score += 1; }
        if has_digit { score += 1; }
        if has_symbol { score += 1; }
        
        // Повторяющиеся символы (штраф)
        let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
        let repetition_ratio = unique_chars as f32 / password.len() as f32;
        if repetition_ratio < 0.7 { score -= 1; }
        
        match score {
            0..=2 => PasswordStrength::Weak,
            3..=5 => PasswordStrength::Medium,
            6..=8 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }

    fn show_password_strength_indicator(&self, ui: &mut egui::Ui, strength: PasswordStrength) {
        ui.horizontal(|ui| {
            ui.label("Сложность:");
            let (color, text) = match strength {
                PasswordStrength::Weak => (egui::Color32::RED, "Слабый"),
                PasswordStrength::Medium => (egui::Color32::YELLOW, "Средний"),
                PasswordStrength::Strong => (egui::Color32::GREEN, "Сильный"),
                PasswordStrength::VeryStrong => (egui::Color32::from_rgb(0, 200, 0), "Очень сильный"),
            };
            ui.colored_label(color, text);
        });
    }

    fn copy_to_clipboard(&mut self, text: &str) {
        use clipboard::{ClipboardContext, ClipboardProvider};
        
        match ClipboardContext::new() {
            Ok(mut ctx) => {
                match ctx.set_contents(text.to_owned()) {
                    Ok(_) => {
                        self.error_message = Some("✅ Скопировано в буфер обмена".to_string());
                    }
                    Err(_) => {
                        self.error_message = Some("❌ Ошибка копирования".to_string());
                    }
                }
            }
            Err(_) => {
                self.error_message = Some("❌ Ошибка доступа к буферу обмена".to_string());
            }
        }
    }

    fn save_entry(&mut self) {
        if self.form_name.is_empty() {
            self.error_message = Some("Введите название записи".to_string());
            return;
        }
        
        let entry = Entry {
            name: self.form_name.clone(),
            username: self.form_username.clone(),
            password: self.form_password.clone(),
        };
        
        match &mut self.vault {
            Some(vault) => {
                match vault.lock() {
                    Ok(mut v) => {
                        match self.state {
                            AppState::AddEntry => {
                                v.add_entry(entry);
                            }
                            AppState::EditEntry(index) => {
                                match v.edit_entry(index, Some(entry.username), Some(entry.password)) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        self.error_message = Some(format!("❌ Ошибка редактирования: {}", e));
                                        return;
                                    }
                                }
                            }
                            _ => {}
                        }
                        // Сохраняем в файл
                        let vault_path = Path::new("data\\vault.enc");
                        match v.save_to_file(vault_path) {
                            Ok(_) => {
                                self.error_message = Some("✅ Запись сохранена".to_string());
                                self.state = AppState::Main;
                                self.selected_entry = None;
                                // Обновляем локальный список
                                self.entries = v.get_entries().clone();
                            }
                            Err(e) => {
                                self.error_message = Some(format!("❌ Ошибка сохранения: {}", e));
                            }
                        }
                    }
                    Err(_) => {
                        self.error_message = Some("❌ Ошибка доступа к хранилищу".to_string());
                    }
                }
            }
            None => {
                self.error_message = Some("❌ Хранилище не загружено".to_string());
            }
        }
    }

    fn delete_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            match &mut self.vault {
                Some(vault) => {
                    match vault.lock() {
                        Ok(mut v) => {
                            v.remove_entry(index);
                            let vault_path = Path::new("data\\vault.enc");
                            match v.save_to_file(vault_path) {
                                Ok(_) => {
                                    self.error_message = Some("✅ Запись удалена".to_string());
                                    // Обновляем локальный список
                                    self.entries = v.get_entries().clone();
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("❌ Ошибка удаления: {}", e));
                                }
                            }
                        }
                        Err(_) => {
                            self.error_message = Some("❌ Ошибка доступа к хранилищу".to_string());
                        }
                    }
                }
                None => {
                    self.error_message = Some("❌ Хранилище не загружено".to_string());
                }
            }
        }
    }

    fn attempt_login(&mut self) {
        if self.master_password.is_empty() {
            self.error_message = Some("Введите мастер-пароль".to_string());
            return;
        }

        match Vault::new(&self.master_password) {
            Ok(mut vault) => {
                let vault_path = Path::new("data\\vault.enc");
                match vault.load_from_file(vault_path) {
                    Ok(_) => {
                        self.vault = Some(Arc::new(Mutex::new(vault)));
                        self.state = AppState::Main;
                        self.error_message = None;
                        // Загружаем записи
                        if let Some(v) = &self.vault {
                            if let Ok(v_locked) = v.lock() {
                                self.entries = v_locked.get_entries().clone();
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Ошибка загрузки: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Ошибка инициализации: {}", e));
            }
        }
    }

    fn create_vault(&mut self) {
        if self.master_password.is_empty() {
            self.error_message = Some("Введите мастер-пароль для создания хранилища".to_string());
            return;
        }

        match Vault::new(&self.master_password) {
            Ok(vault) => {
                // Создаем директорию если её нет
                std::fs::create_dir_all("data").unwrap_or_default();
                
                let vault_path = Path::new("data\\vault.enc");
                match vault.save_to_file(vault_path) {
                    Ok(_) => {
                        self.vault = Some(Arc::new(Mutex::new(vault)));
                        self.state = AppState::Main;
                        self.error_message = None;
                        self.entries = Vec::new();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Ошибка создания хранилища: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Ошибка инициализации: {}", e));
            }
        }
    }
}