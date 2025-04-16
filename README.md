# 🎲 Eng Roulette

**Eng Roulette** — это веб-приложение для случайных разговоров на английском языке. Пользователи подключаются к комнатам через WebRTC в формате "рулетки" и общаются с другими участниками для практики английского.

## 🚀 Функциональность

- Авторизация и учетные записи
- Комнаты с видеосвязью, WebRTC видеочат между участниками
- Интеграция с биллингом (опционально)

## 🛠️ Стек технологий

- **Backend**: Rust (async, axum + websocket, actix, sqlx, webrtc-rs)
- **Frontend**: TypeScript + React
- **Базы данных**: PostgreSQL
- **Инфраструктура**: Docker Compose

## 🌟 О проекте

- 🔐 Авторизация и аутентификация реализованы в сервисе `account` с использованием HS256 JWT-токенов и подхода [гексагональной архитектуры](https://github.com/microsoft/cookiecutter-rust-actix-clean-architecture/blob/main/docs/onion-architecture-article.md). 
Также в `account/src/test` присутствуют модульные тесты для проверки логики + настроен Github CI.
- 📡 Сервис `room` содержит WebRTC SFU (Selective Forwarding Unit) — сервер видеоконференций, написанный на базе библиотеки [webrtc-rs](https://github.com/webrtc-rs/webrtc), поддерживающий несколько участников в одной комнате.
- 🦀 Идиоматичный Rust-код: проект написан с соблюдением лучших практик Rust — строгая типизация, `anyhow::Result`, `thiserror`, pattern matching, кастомные extractors для Axum и читаемая архитектура без избыточной абстракции.
- 📋 Структурированные логи через `env_logger`
- 🌍 Dev-инфраструктура: проект можно запускать локально через туннель с HTTPS-доступом через публичный сервер. Удобно для тестирования, демонстраций и интеграции с внешними сервисами.

## 🧱 Структура проекта

```
.
├── account/        # Сервис авторизации и управления пользователями
├── room/           # Сервис видео комнат с WebRTC
├── frontend/       # React SPA
├── scripts/        # Вспомогательные скрипты
```

## 📦 Запуск проекта

1. Скопируйте `.env.example` в `.env` и отредактируйте переменные.
2. Сгенерируйте env `SECRET_KEY`
```bash
SECRET_KEY=$(openssl rand -hex 32)
echo -e "\SECRET_KEY=${SECRET_KEY}" >> .env
```
3. Создайте базу данные `CREATE DATABASE roulette;` и выполните `account/init.sql`
4. Запустите контейнеры:

```bash
docker-compose up postgres
cargo run --bin account # запускаем сервис account
cargo run --bin room # запускаем сервис room
VITE_ACCOUNT_API=http://localhost:8081 VITE_ROOM_API=http://localhost:8082 npm run dev # запускаем сервис react spa
```
Frontend будет доступен на http://localhost:3000

> ⚠️ Важно: Для работы видеосвязи в браузере требуется HTTPS. Браузеры (Chrome, Firefox и др.) блокируют доступ к камере и микрофону на сайтах без HTTPS.
В scripts/compose.yaml уже включены сервисы traefik, proxy и tunnel, чтобы можно было запускать dev-проект на сервере с поддержкой HTTPS через скрипт tunnel.sh.

## 🌐 Запуск через туннель с HTTPS через публичный сервер

Для организации HTTPS-соединения и публичного доступа к dev-проекту можно использовать связку tunnel.sh + публичный traefik сервер:

1. На сервере:
    - Скопируйте содержимое директории scripts/ и файл compose.yaml на публичный сервер. 
    - Измените `domains` и `email` в `traefik.yml`, поменяйте адрес `botenza.org` по умолчанию на свой
    - Выполните команду:

```bash
SERVER=myserver.ru docker compose up -d
```
Это поднимет сервисы traefik, proxy и tunnel, необходимые для туннеля и HTTPS.

2. Локально:
    - Запустите tunnel
```bash
make tunnel SERVER=myserver.ru
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