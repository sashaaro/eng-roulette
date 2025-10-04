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

## Запуск через туннель с HTTPS через публичный сервер

Для организации HTTPS-соединения и публичного доступа к dev-проекту можно использовать связку `make tunnel` + публичный Caddy сервер:

Добавьте на сервере `example.com` в `Caddyfile`
```caddyfile
webrtc.example.com {
    reverse_proxy localhost:8083
}
```
На сервере `example.com` запустите туннель:

```bash
docker compose up tunnel
```

С работающими сервисами еще локально запустите api gateway `caddy` и `tunnel`
```bash
docker compose up caddy
make SERVER=example.com tunnel
```

Этот скрипт установит безопасный туннель между локальным окружением и сервером.
Теперь ваш локальный проект будет доступен по HTTPS через домен, настроенный на сервере.

## Планы

- Подбор случайного собеседника
- Рейтинговая система
- Интеграция с AI (например, для подсказок и переводов)
- Интеграция с биллингом (опционально)
