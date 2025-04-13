# 🎲 Eng Roulette

**Eng Roulette** — это веб-приложение для случайных разговоров на английском языке. Пользователи подключаются к комнатам через WebRTC в формате "рулетки" и общаются с другими участниками для практики английского.

## 🚀 Функциональность

- Авторизация и учетные записи
- Комнаты с видеосвязью, видеочат между участниками
- Интеграция с биллингом (опционально)

## 🛠️ Стек технологий

- **Backend**: Rust (async, Axum, Actix)
- **Frontend**: TypeScript + React
- **Базы данных**: PostgreSQL
- **Инфраструктура**: Docker Compose

## 🌟 Особенности проекта

- 🔐 Авторизация и аутентификация реализованы в сервисе `account` с использованием JWT-токенов и подхода гексагональной архитектуры. 
Также в `account/src/test` присутствуют модульные тесты для проверки логики.

- 📡 Модуль `room/src/webrtc/` содержит SFU (Selective Forwarding Unit) — сервер видеоконференций, написанный на базе библиотеки `webrtc-rs`, поддерживающий несколько участников в одной комнате.

- 🔍 `room/src/webrtc/extract.rs` — кастомный Axum extractor, позволяющий удобно извлекать JWT payload в обработчиках:

```rust
async fn accept_offer(
JWT(claims): JWT,
...
) -> Result<impl IntoResponse, AppError> {}
```
- 🛠️ Используются современные подходы обработки ошибок: `anyhow` и `thiserror`
- 📋 Структурированные логи через env_logger
- 🗄️ Работа с базой данных реализована через `sqlx::Pool<Postgres>` с полной асинхронностью и безопасной типизацией SQL-запросов

## 🧱 Структура проекта

```
.
├── account/        # Сервис авторизации и управления пользователями
├── billing/        # Сервис биллинг (in progress)
├── room/           # Webrtc sfu сервис, управления комнатами
├── frontend/       # React SPA
├── scripts/        # Вспомогательные скрипты
├── compose.yaml    # Конфигурация Docker Compose
├── Cargo.toml      # Зависимости Rust
```

## 📦 Запуск проекта

1. Скопируйте `.env.tmp` в `.env` и отредактируйте переменные.
2. Запустите контейнеры:

```bash
docker-compose up postgres
cargo run --bin account # запускаем сервис account
cargo run --bin room # запускаем сервис room
npm run dev # запускаем сервис react spa
```
Frontend будет доступен на http://localhost:3000

> ⚠️ Важно: Для работы видеосвязи в браузере требуется HTTPS. Браузеры (Chrome, Firefox и др.) блокируют доступ к камере и микрофону на сайтах без HTTPS.
В scripts/compose.yaml уже включены сервисы traefik, proxy и tunnel, чтобы можно было запускать dev-проект на сервере с поддержкой HTTPS через скрипт tunnel.sh.

## 🌐 Запуск через туннель с HTTPS через публичный сервер

Для организации HTTPS-соединения и публичного доступа к dev-проекту можно использовать связку tunnel.sh + публичный сервер с Docker Compose:

1. На сервере:
    - Скопируйте содержимое директории scripts/ и файл compose.yaml на публичный сервер. 
    - Выполните команду:

```bash
docker compose -f compose.yaml up -d
```
Это поднимет сервисы traefik, proxy и tunnel, необходимые для туннеля и HTTPS.

2. Локально:
    - Запустите ./tunnel.sh
```bash
./tunnel.sh
```

Этот скрипт установит безопасный туннель между локальным окружением и сервером.
Теперь ваш локальный проект будет доступен по HTTPS через домен, настроенный на сервере.

## 📌 Планы

- Подбор случайного собеседника
- Рейтинговая система
- Интеграция с AI (например, для подсказок и переводов)
- Интеграция с биллингом (опционально)

## 📄 Лицензия
MIT