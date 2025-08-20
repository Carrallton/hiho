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
    /// Копировать пароль в буфер обмена
    Copy {
        /// Название сервиса или номер записи
        name_or_index: String,
    },
    /// Удалить запись
    Remove {
        /// Название сервиса или номер записи
        name_or_index: String,
    },
    /// Поиск записей по части имени
    Search {
        /// Часть названия для поиска
        query: String,
    },
    /// Редактировать запись
    Edit {
        /// Название сервиса или номер записи
        name_or_index: String,
        /// Новое имя пользователя
        #[arg(short, long)]
        username: Option<String>,
        /// Новый пароль
        #[arg(short, long)]
        password: Option<String>,
        /// Длина нового пароля (если генерируем)
        #[arg(long, default_value = "16")]
        length: usize,
    },
    /// Экспорт данных в файл
    Export {
        /// Путь к файлу для экспорта
        #[arg(short, long, default_value = "hiho_export.json")]
        file: String,
        
        /// Формат экспорта (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Импорт данных из файла
    Import {
        /// Путь к файлу для импорта
        #[arg(short, long)]
        file: String,
        
        /// Формат импорта (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}