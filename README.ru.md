# Rust Библиотека для проверки статуса Minecraft серверов

[![Crates.io](https://img.shields.io/crates/v/rust-mc-status)](https://crates.io/crates/rust-mc-status)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Высокопроизводительная асинхронная библиотека на Rust для получения статуса серверов Minecraft Java Edition и Bedrock Edition.

## Возможности

*   **Поддержка двух протоколов**: Пинг серверов Minecraft Java Edition (`25565`) и Bedrock Edition (`19132`).
*   **Асинхронность**: Построена на Tokio для неблокирующих операций и высокой конкурентности.
*   **Массовые запросы**: Параллельный пинг множества серверов с настраиваемым лимитом.
*   **Кэширование DNS**: Автоматическое кэширование DNS-запросов для снижения задержки.
*   **Структурированные данные**: Возвращает детальные, сериализуемые (через `serde`) данные, включая информацию о версии, количество игроков, MOTD и список игроков онлайн.
*   **Работа с иконками**: Удобное получение и сохранение иконки сервера (только для Java Edition).
*   **Надежная обработка ошибок**: Comprehensive error types using `thiserror`.

## Установка

Добавьте в ваш `Cargo.toml`:

```toml
[dependencies]
rust-mc-status = "1.0.1"
tokio = { version = "1", features = ["full"] }
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

    let servers = vec![
        ServerInfo {
            address: "mc.hypixel.net:25565".to_string(),
            edition: ServerEdition::Java,
        },
        ServerInfo {
            address: "geo.hivebedrock.network:19132".to_string(),
            edition: ServerEdition::Bedrock,
        },
    ];

    let results = client.ping_many(&servers).await;

    for (server, result) in results {
        println!("\nСервер: {}", server.address);
        match result {
            Ok(status) => {
                println!("Статус: Онлайн ({} мс)", status.latency);
                // ... работа с данными статуса (Java или Bedrock)
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
    Ok(())
}
```

### Ключевые структуры и методы

*   `McClient`: Основной клиент для выполнения запросов.
    *   `new()`, `with_timeout()`, `with_max_parallel()`
    *   `ping(address, edition)`: Пинг одного сервера.
    *   `ping_many(servers)`: Пинг нескольких серверов параллельно.
*   `ServerStatus`: Результат успешного запроса.
    *   `online`: `bool`
    *   `latency`: `f64` (задержка)
    *   `data`: `ServerData` (перечисление, содержит либо `JavaStatus`, либо `BedrockStatus`)
*   `JavaStatus`: Содержит детальную информацию от Java сервера.
    *   `save_favicon(filename)`: Сохраняет иконку сервера в формате PNG из base64.

## Лицензия

Этот проект лицензирован под MIT License - подробности в файле [LICENSE](LICENSE).