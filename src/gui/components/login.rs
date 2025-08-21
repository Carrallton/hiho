use eframe::egui;
use crate::gui::app::{HihoApp, AppState};
use crate::vault::Vault;
use std::sync::{Arc, Mutex};

pub struct LoginScreen;

impl LoginScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, _ctx: &egui::Context, ui: &mut egui::Ui, app: &mut HihoApp) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("🔐 Вход в hiho");
            ui.add_space(30.0);
            
            ui.horizontal(|ui| {
                ui.label("🔑 Мастер-пароль:");
                let password_field = ui.add(
                    egui::TextEdit::singleline(&mut app.master_password)
                        .password(true)
                        .desired_width(200.0)
                );
                
                if password_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.attempt_login(app);
                }
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("🔓 Войти").clicked() {
                    self.attempt_login(app);
                }
                
                if ui.button("🆕 Создать хранилище").clicked() {
                    self.create_vault(app);
                }
            });
        });
    }

    fn attempt_login(&self, app: &mut HihoApp) {
        if app.master_password.is_empty() {
            app.error_message = Some("Введите мастер-пароль".to_string());
            return;
        }

        match Vault::new(&app.master_password) {
            Ok(mut vault) => {
                match vault.load_from_file(std::path::Path::new("data\\vault.enc")) {
                    Ok(_) => {
                        app.vault = Some(Arc::new(Mutex::new(vault)));
                        app.state = AppState::Main;
                        app.error_message = None;
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Ошибка загрузки: {}", e));
                    }
                }
            }
            Err(e) => {
                app.error_message = Some(format!("Ошибка: {}", e));
            }
        }
    }

    fn create_vault(&self, app: &mut HihoApp) {
        if app.master_password.is_empty() {
            app.error_message = Some("Введите мастер-пароль для создания хранилища".to_string());
            return;
        }

        match Vault::new(&app.master_password) {
            Ok(vault) => {
                // Создаем директорию если её нет
                std::fs::create_dir_all("data").unwrap_or_default();
                
                match vault.save_to_file(std::path::Path::new("data\\vault.enc")) {
                    Ok(_) => {
                        app.vault = Some(Arc::new(Mutex::new(vault)));
                        app.state = AppState::Main;
                        app.error_message = None;
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Ошибка создания хранилища: {}", e));
                    }
                }
            }
            Err(e) => {
                app.error_message = Some(format!("Ошибка: {}", e));
            }
        }
    }
}