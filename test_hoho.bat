@echo off
chcp 65001 >nul
title Тестирование hiho CLI

cls
echo ╔════════════════════════════════════════════════╗
echo ║           🧪 ТЕСТИРОВАНИЕ HIHO CLI            ║
echo ╚════════════════════════════════════════════════╝
echo.

:: Очистка предыдущих данных
if exist data rmdir /s /q data >nul
if exist test_results.txt del test_results.txt >nul

set TEST_PASSWORD=TestMasterPassword123!
set TEST_COUNT=0
set PASS_COUNT=0

:: Функция для логгирования
:log_result
if "%~1"=="PASS" (
    set /a PASS_COUNT+=1
    echo ✅ %~2
    echo ✅ %~2 >> test_results.txt
) else (
    echo ❌ %~2
    echo ❌ %~2 >> test_results.txt
)
set /a TEST_COUNT+=1
goto :eof

:: Тест 1: Инициализация
echo 🔐 Тест 1: Инициализация хранилища
echo %TEST_PASSWORD% | hiho.exe init >nul 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Инициализация хранилища"
) else (
    call :log_result FAIL "Инициализация хранилища"
)

:: Тест 2: Добавление записей
echo ➕ Тест 2: Добавление записей
echo %TEST_PASSWORD% | hiho.exe add -n "github.com" -u "dev@github.com" -p "MyGitHubPass123!" >nul 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Добавление записи github.com"
) else (
    call :log_result FAIL "Добавление записи github.com"
)

echo %TEST_PASSWORD% | hiho.exe add -n "google.com" -u "user@gmail.com" --length 16 >nul 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Добавление записи google.com с автогенерацией"
) else (
    call :log_result FAIL "Добавление записи google.com с автогенерацией"
)

:: Тест 3: Просмотр записей
echo 📋 Тест 3: Просмотр записей
echo %TEST_PASSWORD% | hiho.exe list > test_output.txt 2>&1
findstr /C:"github.com" test_output.txt >nul
if %errorlevel% equ 0 (
    call :log_result PASS "Просмотр записей"
) else (
    call :log_result FAIL "Просмотр записей"
)

:: Тест 4: Поиск записей
echo 🔍 Тест 4: Поиск записей
echo %TEST_PASSWORD% | hiho.exe search "git" > test_output.txt 2>&1
findstr /C:"github.com" test_output.txt >nul
if %errorlevel% equ 0 (
    call :log_result PASS "Поиск записей"
) else (
    call :log_result FAIL "Поиск записей"
)

:: Тест 5: Генерация паролей
echo 🔤 Тест 5: Генерация паролей
hiho.exe generate --length 12 > test_output.txt 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Генерация простого пароля"
) else (
    call :log_result FAIL "Генерация простого пароля"
)

hiho.exe generate --length 16 --secure > test_output.txt 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Генерация безопасного пароля"
) else (
    call :log_result FAIL "Генерация безопасного пароля"
)

:: Тест 6: Экспорт данных
echo 📤 Тест 6: Экспорт данных
echo %TEST_PASSWORD% | hiho.exe export --file test_export.json >nul 2>&1
if exist test_export.json (
    call :log_result PASS "Экспорт в JSON"
) else (
    call :log_result FAIL "Экспорт в JSON"
)

:: Тест 7: Конфигурация автоблокировки
echo ⏰ Тест 7: Конфигурация автоблокировки
echo %TEST_PASSWORD% | hiho.exe lock-config --timeout 5 >nul 2>&1
if %errorlevel% equ 0 (
    call :log_result PASS "Настройка автоблокировки"
) else (
    call :log_result FAIL "Настройка автоблокировки"
)

:: Вывод результатов
echo.
echo ╔════════════════════════════════════════════════╗
echo ║              📊 РЕЗУЛЬТАТЫ ТЕСТОВ              ║
echo ╚════════════════════════════════════════════════╝
echo Пройдено: %PASS_COUNT%/%TEST_COUNT%
echo.

if %PASS_COUNT% equ %TEST_COUNT% (
    echo 🎉 Все тесты пройдены успешно!
) else (
    echo ⚠️  Некоторые тесты не пройдены
)

echo.
echo 📁 Тестовые данные сохранены в папке test_hiho
echo 📄 Подробные результаты в test_results.txt
echo.

pause