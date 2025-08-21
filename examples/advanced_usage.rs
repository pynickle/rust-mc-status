//! Example of advanced usage of the rust-mc-status library
//! Demonstrates new capabilities of the library

use rust_mc_status::{McClient, ServerEdition, ServerInfo};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = McClient::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_parallel(10);

    // List of servers to check
    let servers = vec![
        ServerInfo {
            address: "mc.hypixel.net".to_string(),
            edition: ServerEdition::Java,
        },
        ServerInfo {
            address: "geo.hivebedrock.network:19132".to_string(),
            edition: ServerEdition::Bedrock,
        },
        ServerInfo {
            address: "localhost:25565".to_string(),
            edition: ServerEdition::Java,
        },
    ];

    println!("Requesting status for {} servers...", servers.len());
    let results = client.ping_many(&servers).await;

    for (server, result) in results {
        println!("\n{}", "=".repeat(50));
        println!("Server: {} ({:?})", server.address, server.edition);

        match result {
            Ok(status) => {
                println!("Status: ✅ Online (latency: {:.2} ms)", status.latency);
                println!("IP: {}, Port: {}", status.ip, status.port);
                println!("Hostname: {}", status.hostname);

                // DNS information
                if let Some(dns) = status.dns {
                    println!("DNS: A-records: {:?}, CNAME: {:?}", dns.a_records, dns.cname);
                }

                // Processing data depending on server type
                match status.data {
                    rust_mc_status::ServerData::Java(java_status) => {
                        println!("Version: {} (protocol: {})", java_status.version.name, java_status.version.protocol);
                        println!("Players: {}/{}", java_status.players.online, java_status.players.max);
                        println!("Description: {}", java_status.description);

                        // Используем ссылку вместо перемещения
                        if let Some(ref map) = java_status.map {
                            println!("Map: {}", map);
                        }

                        if let Some(ref gamemode) = java_status.gamemode {
                            println!("Game mode: {}", gamemode);
                        }

                        if let Some(ref software) = java_status.software {
                            println!("Server software: {}", software);
                        }

                        // Plugins and mods
                        if let Some(ref plugins) = java_status.plugins {
                            println!("Plugins ({}):", plugins.len());
                            for plugin in plugins.iter().take(5) {
                                println!("  - {} {}", plugin.name, plugin.version.as_deref().unwrap_or(""));
                            }
                            if plugins.len() > 5 {
                                println!("  ... and {} more", plugins.len() - 5);
                            }
                        }

                        if let Some(ref mods) = java_status.mods {
                            println!("Mods ({}):", mods.len());
                            for mod_ in mods.iter().take(5) {
                                println!("  - {} {}", mod_.modid, mod_.version.as_deref().unwrap_or(""));
                            }
                            if mods.len() > 5 {
                                println!("  ... and {} more", mods.len() - 5);
                            }
                        }

                        // Saving icon
                        if let Some(ref _favicon) = java_status.favicon {
                            if let Err(e) = java_status.save_favicon("server_icon.png") {
                                println!("Failed to save icon: {}", e);
                            } else {
                                println!("Icon saved as server_icon.png");
                            }
                        }
                    },
                    rust_mc_status::ServerData::Bedrock(bedrock_status) => {
                        println!("Edition: {}", bedrock_status.edition);
                        println!("Version: {}", bedrock_status.version);
                        println!("Players: {}/{}", bedrock_status.online_players, bedrock_status.max_players);
                        println!("MOTD: {}", bedrock_status.motd);

                        if let Some(ref map) = bedrock_status.map {
                            println!("Map: {}", map);
                        }

                        if let Some(ref software) = bedrock_status.software {
                            println!("Server software: {}", software);
                        }

                        println!("Game mode: {} ({})", bedrock_status.game_mode, bedrock_status.game_mode_numeric);
                    }
                }
            }
            Err(e) => {
                println!("Status: ❌ Error: {}", e);
            }
        }
    }

    Ok(())
}