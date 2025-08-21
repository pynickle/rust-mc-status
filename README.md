# Rust Minecraft Server Status Library

[![Crates.io](https://img.shields.io/crates/v/rust-mc-status)](https://crates.io/crates/rust-mc-status)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance, asynchronous Rust library for querying the status of both Minecraft Java Edition and Bedrock Edition servers.

## Features

*   **Dual Protocol Support**: Ping both Minecraft Java Edition (`25565`) and Bedrock Edition (`19132`) servers.
*   **Async/Await**: Built on Tokio for non-blocking operations and high concurrency.
*   **Batch Queries**: Ping multiple servers in parallel with configurable concurrency limits.
*   **DNS Caching**: Automatically caches DNS lookups to reduce latency for repeated queries.
*   **Structured Data**: Returns richly structured, serializable data (using `serde`), including version info, player counts, MOTD, and sample players.
*   **Favicon Handling**: Easily retrieve and save the server's favicon (Java Edition only).
*   **Robust Error Handling**: Comprehensive error types using `thiserror`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-mc-status = "1.0.1"
tokio = { version = "*", features = ["full"] }
```

## Usage

### Basic Example

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
        println!("\nServer: {}", server.address);
        match result {
            Ok(status) => {
                println!("Status: Online ({} ms)", status.latency);
                // ... work with the status data (Java or Bedrock)
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    Ok(())
}
```

### Key Structs and Methods

*   `McClient`: The main client for making requests.
    *   `new()`, `with_timeout()`, `with_max_parallel()`
    *   `ping(address, edition)`: Ping a single server.
    *   `ping_many(servers)`: Ping multiple servers in parallel.
*   `ServerStatus`: The result of a successful ping.
    *   `online`: `bool`
    *   `latency`: `f64`
    *   `data`: `ServerData` (enum containing either `JavaStatus` or `BedrockStatus`)
*   `JavaStatus`: Contains detailed information from a Java server.
    *   `save_favicon(filename)`: Saves the base64-encoded favicon to a PNG file.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
