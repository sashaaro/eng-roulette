# 🎲 Eng Roulette

**Eng Roulette** — это веб-приложение для случайных разговоров на английском языке. Пользователи подключаются к комнатам через WebRTC в формате "рулетки" и общаются с другими участниками для практики английского.

- Авторизация по никнейму и Google OAuth
- Комнаты с видеосвязью, WebRTC видеочат между участниками
- Интеграция с биллингом (опционально)

## Стек технологий

- **Backend**: Rust (axum + websocket, actix, sqlx, webrtc-rs)
- **Frontend**: TypeScript + React
- **Базы данных**: PostgreSQL
- **Инфраструктура**: Docker Compose + Traefik API Gateway + Github Actions CI (проверка cargo fmt, clippy, test)

## О проекте

- Авторизация и аутентификация реализованы в сервисе `account` с использованием HS256 JWT-токенов и подхода [гексагональной архитектуры](https://github.com/microsoft/cookiecutter-rust-actix-clean-architecture/blob/main/docs/onion-architecture-article.md). 
- Присутствуют интеграционные и unit-тесты с использованием библиотеки моков [mockall](https://crates.io/crates/mockall). Запуск через `make test`
- Сервис `room` содержит **собственный WebRTC SFU** (Selective Forwarding Unit) — сервис видеоконференций, написанный на базе библиотеки [webrtc-rs](https://crates.io/crates/webrtc), поддерживающий несколько участников в одной комнате.
- Идиоматичный Rust-код: `Result`, `?`, `anyhow`, `thiserror`, pattern matching, кастомные `extractors` для `Axum`.
- Структурированные логи через `env_logger`
- Dev-инфраструктура: проект можно запускать локально через туннель с HTTPS-доступом через публичный API Gateway сервер. Удобно для тестирования, демонстраций и интеграции с внешними сервисами.

## Структура проекта

```
.
├── account/        # Сервис авторизации и управления пользователями на Actix
    ├── src/
    ├── tests/       # Интеграционные тесты
    ├── ...
├── room/           # Сервис видео комнат с WebRTC на Axum + Websocket + WebRTC
├── frontend/       # React SPA
├── scripts/        # Вспомогательные скрипты
```

## Запуск проекта

1. Скопируйте `.env.example` в `.env` и отредактируйте переменные.
2. Сгенерируйте и добавьте env `SECRET_KEY` для JWT-токенов с помощью команды:
```bash
SECRET_KEY=$(openssl rand -hex 32)
echo -e "SECRET_KEY=${SECRET_KEY}" >> .env
```
3. Добавьте переменные `OAUTH_GOOGLE_CLIENT_ID` и `OAUTH_GOOGLE_CLIENT_SECRET` в `.env`
4. 
4. Запустите контейнер `postgres`, выполните `account/init.sql`, запустите сервис `account` и `room`, а затем `frontend`:

```bash
docker-compose up postgres
cargo run --bin account # запускаем сервис account
cargo run --bin room # запускаем сервис room
npm run dev # запускаем сервис react spa
```
Frontend будет доступен на http://localhost:5173

> ⚠️ Важно: Для корректной работы видеосвязи в браузере может потребоваться HTTPS. Браузеры (Chrome, Firefox и др.) могут блокировать доступ к камере и микрофону на сайтах без HTTPS.
В scripts/compose.yaml уже включены сервисы traefik, proxy и tunnel, чтобы можно было запускать dev-проект на сервере с поддержкой HTTPS через скрипт tunnel.sh.

## Запуск через туннель с HTTPS через публичный сервер

Для организации HTTPS-соединения и публичного доступа к dev-проекту можно использовать связку `make tunnel` + публичный traefik сервер:

1. На сервере `example.com`:
    - Скопируйте содержимое директории scripts/ и файл compose.yaml на публичный сервер. 
    - Измените `domains` и `email` в `traefik.yml`, поменяйте адрес `botenza.org` по умолчанию на свой
    - Выполните команду:

```bash
SERVER=example.com docker compose up -d
```
Это поднимет сервисы traefik, proxy и tunnel, необходимые для туннеля и HTTPS.

2. Локально:
    - в папке `frontend` выполните команду `cp .env.example .env`
    - в `.env` добавьте `VITE_ACCOUNT_API=https://example.com/api/account` и `VITE_ROOM_API=https://example.com/api/room`
    - Запустите tunnel и frontend
```bash
make SERVER=example.com tunnel
npm run dev
```

Этот скрипт установит безопасный туннель между локальным окружением и сервером.
Теперь ваш локальный проект будет доступен по HTTPS через домен, настроенный на сервере.

## Планы

- Подбор случайного собеседника
- Рейтинговая система
- Интеграция с AI (например, для подсказок и переводов)
- Интеграция с биллингом (опционально)
