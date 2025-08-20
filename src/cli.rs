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
        /// Пароль (если не указан, будет сгенерирован)
        #[arg(short, long)]
        password: Option<String>,
        
        /// Длина пароля (если генерируем)
        #[arg(long, default_value = "16")]
        length: usize,
    },
    /// Показать все записи
    List,
    /// Инициализировать хранилище
    Init,
    /// Сгенерировать пароль
    Generate {
        /// Длина пароля
        #[arg(short, long, default_value = "16")]
        length: usize,
        
        /// Использовать специальные символы
        #[arg(short, long)]
        secure: bool,
    },
}