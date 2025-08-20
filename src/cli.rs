use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hiho")]
#[command(about = "Менеджер паролей уровня NSA", version = "0.1")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Добавить новую запись
    Add {
        /// Название сервиса
        #[arg(short, long)]
        name: String,
        /// Имя пользователя
        #[arg(short, long)]
        username: String,
        /// Пароль
        #[arg(short, long)]
        password: String,
    },
    /// Показать все записи
    List,
    /// Инициализировать хранилище
    Init,
}