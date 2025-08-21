# Rust библиотека для проверки статуса серверов Minecraft

[![Crates.io](https://img.shields.io/crates/v/rust-mc-status)](https://crates.io/crates/rust-mc-status)
[![Documentation](https://docs.rs/rust-mc-status/badge.svg)](https://docs.rs/rust-mc-status)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Высокопроизводительная асинхронная Rust библиотека для проверки статуса серверов Minecraft Java Edition и Bedrock Edition.

## Возможности

*   **Поддержка двух протоколов**: Пинг серверов Minecraft Java Edition (`25565`) и Bedrock Edition (`19132`).
*   **Асинхронность**: Построена на Tokio для неблокирующих операций и высокой параллельности.
*   **Массовые запросы**: Параллельная проверка множества серверов с настраиваемыми лимитами.
*   **Кэширование DNS**: Автоматическое кэширование DNS запросов для уменьшения задержек.
*   **Структурированные данные**: Возвращает богатые структурированные, сериализуемые данные (с использованием `serde`), включая информацию о версии, количество игроков, MOTD, карту, режим игры, плагины, моды и многое другое.
*   **Работа с иконками**: Удобное получение и сохранение иконки сервера (только Java Edition).
*   **Надежная обработка ошибок**: Полноценные типы ошибок с использованием `thiserror`.
*   **Расширенная информация**: Подробные данные о плагинах, модах, DNS и другом.

## Установка

Добавьте в ваш `Cargo.toml`:

```toml
[dependencies]
rust-mc-status = "1.1.0"
tokio = { version = "*", features = ["full"] }
```

## Использование

### Базовый пример

```rust
use rust_mc_status::{McClient, ServerEdition, ServerInfo};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = McClient::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_parallel(10);

    // Проверка одного сервера
    let status = client.ping("mc.hypixel.net", ServerEdition::Java).await?;
    println!("Статус: {:?}", status);

    // Массовая проверка серверов
    let servers = vec![
        ServerInfo {
            address: "mc.hypixel.net".to_string(),
            edition: ServerEdition::Java,
        },
        ServerInfo {
            address: "geo.hivebedrock.network:19132".to_string(),
            edition: ServerEdition::Bedrock,
        },
    ];

    let results = client.ping_many(&servers).await;

    for (server, result) in results {
        println!("Сервер: {} - {:?}", server.address, result);
    }
    
    Ok(())
}
```

### Расширенный пример

См. [examples/advanced_usage.rs](examples/advanced_usage.rs) для демонстрации всех новых возможностей библиотеки.

## Основные структуры и методы

*   `McClient`: Основной клиент для выполнения запросов.
    *   `new()`, `with_timeout()`, `with_max_parallel()`
    *   `ping(address, edition)`: Проверить один сервер.
    *   `ping_many(servers)`: Проверить несколько серверов параллельно.
*   `ServerStatus`: Результат успешного запроса.
    *   `online`: `bool`
    *   `ip`: `String` - IP адрес сервера
    *   `port`: `u16` - Порт сервера
    *   `hostname`: `String` - Имя хоста
    *   `latency`: `f64` - Задержка в мс
    *   `dns`: `Option<DnsInfo>` - DNS информация
    *   `data`: `ServerData` (enum, содержит либо `JavaStatus`, либо `BedrockStatus`)
*   `JavaStatus`: Содержит подробную информацию от Java сервера.
    *   `version`: Информация о версии
    *   `players`: Информация об игроках
    *   `description`: Описание сервера (MOTD)
    *   `map`: Название карты
    *   `gamemode`: Режим игры
    *   `software`: ПО сервера
    *   `plugins`: Список плагинов
    *   `mods`: Список модов
    *   `save_favicon(filename)`: Сохраняет иконку сервера в PNG файл.
*   `BedrockStatus`: Содержит информацию от Bedrock сервера.
    *   `edition`: Издание Minecraft
    *   `motd`: Сообщение дня
    *   `version`: Версия сервера
    *   `online_players`: Онлайн игроков
    *   `max_players`: Максимум игроков
    *   `map`: Название карты
    *   `software`: ПО сервера
    *   `game_mode`: Режим игры

## Лицензия

Этот проект лицензирован по лицензии MIT - подробности в файле [LICENSE](LICENSE).

## Версия

Рекомендую увеличить версию до **1.1.0** в `Cargo.toml`, так как добавлена новая функциональность без обратной несовместимости:

```toml
[package]
name = "rust-mc-status"
version = "1.1.0"  # Измените с 1.0.3 на 1.1.0
# ... остальное без изменений
```