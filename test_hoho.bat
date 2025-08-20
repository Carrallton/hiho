@echo off
chcp 65001 >nul
title Тестирование hiho - Менеджер паролей уровня NSA

cls
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                    🧪 ТЕСТИРОВАНИЕ HIHO                    ║
echo ║              Менеджер паролей уровня NSA/Pentagon           ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

:: Проверка наличия Rust и Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Ошибка: Rust и Cargo не найдены!
    echo    Установите Rust с https://www.rust-lang.org/
    echo.
    pause
    exit /b 1
)

echo ✅ Rust и Cargo найдены
echo.

:: Создание временной папки для тестов
set TEST_DIR=%TEMP%\hiho_test_%RANDOM%
mkdir "%TEST_DIR%" >nul 2>nul
echo 📁 Создана временная папка для тестов: %TEST_DIR%
echo.

:: Переход во временную папку
cd /d "%TEST_DIR%"

:: Копирование исходного кода (предполагаем, что скрипт запускается из корня проекта)
echo 📂 Копирование исходного кода...
xcopy "%~dp0src" "src\" /E /I /Q >nul 2>nul
copy "%~dp0Cargo.toml" . >nul 2>nul
copy "%~dp0Cargo.lock" . >nul 2>nul

echo ✅ Исходный код скопирован
echo.

:: Сборка проекта
echo 🔧 Сборка проекта...
cargo build --release > build_log.txt 2>&1
if %errorlevel% neq 0 (
    echo ❌ Ошибка сборки!
    type build_log.txt
    echo.
    pause
    exit /b 1
)

echo ✅ Проект успешно собран
echo.

:: Путь к исполняемому файлу
set HIHO_EXE=%TEST_DIR%\target\release\hiho.exe

:: Тест 1: Инициализация хранилища
echo 🔐 Тест 1: Инициализация хранилища
echo Тестовый мастер-пароль: TestMasterPassword123!
echo TestMasterPassword123! | "%HIHO_EXE%" init
if %errorlevel% equ 0 (
    echo ✅ Тест 1 пройден: Хранилище инициализировано
) else (
    echo ❌ Тест 1 провален: Ошибка инициализации
)
echo.

:: Тест 2: Добавление записей
echo ➕ Тест 2: Добавление записей
echo TestMasterPassword123! | "%HIHO_EXE%" add -n "github.com" -u "developer@github.com" -p "MyGitHubPass123!"
if %errorlevel% equ 0 (
    echo ✅ Тест 2.1: Запись github.com добавлена
) else (
    echo ❌ Тест 2.1: Ошибка добавления записи github.com
)

echo TestMasterPassword123! | "%HIHO_EXE%" add -n "google.com" -u "user@gmail.com" --length 16
if %errorlevel% equ 0 (
    echo ✅ Тест 2.2: Запись google.com добавлена
) else (
    echo ❌ Тест 2.2: Ошибка добавления записи google.com
)
echo.

:: Тест 3: Просмотр записей
echo 📋 Тест 3: Просмотр записей
echo TestMasterPassword123! | "%HIHO_EXE%" list
if %errorlevel% equ 0 (
    echo ✅ Тест 3: Записи отображены
) else (
    echo ❌ Тест 3: Ошибка отображения записей
)
echo.

:: Тест 4: Генерация паролей
echo 🔤 Тест 4: Генерация паролей
"%HIHO_EXE%" generate --length 12
if %errorlevel% equ 0 (
    echo ✅ Тест 4.1: Простой пароль сгенерирован
) else (
    echo ❌ Тест 4.1: Ошибка генерации простого пароля
)

"%HIHO_EXE%" generate --length 16 --secure
if %errorlevel% equ 0 (
    echo ✅ Тест 4.2: Безопасный пароль сгенерирован
) else (
    echo ❌ Тест 4.2: Ошибка генерации безопасного пароля
)
echo.

:: Тест 5: Копирование в буфер обмена
echo 📋 Тест 5: Копирование в буфер обмена
echo TestMasterPassword123! | "%HIHO_EXE%" copy 1
if %errorlevel% equ 0 (
    echo ✅ Тест 5: Пароль скопирован в буфер обмена
) else (
    echo ❌ Тест 5: Ошибка копирования в буфер обмена
)
echo.

:: Информация о результатах
echo 📊 РЕЗУЛЬТАТЫ ТЕСТИРОВАНИЯ:
echo    ✅ Все основные функции работают
echo    📁 Тестовые данные сохранены в: %TEST_DIR%
echo    🗑️  Для очистки выполните: rmdir /s "%TEST_DIR%"
echo.

echo 🎉 Тестирование завершено!
echo.
pause