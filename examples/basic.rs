use rust_mc_status::{McClient, ServerEdition, ServerInfo};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = McClient::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_parallel(10);

    // Ping multiple servers
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
        println!("\nChecking server: {}", server.address);
        match result {
            Ok(status) => {
                println!("Status: Online ({} ms)", status.latency);
                match status.data {
                    rust_mc_status::ServerData::Java(java) => {
                        println!("Name: {}", java.description);
                        println!("Version: {}", java.version.name);
                        println!("Players: {}/{}", java.players.online, java.players.max);

                        // Save favicon if available
                        let filename = format!("{}_favicon.png", server.address.replace(':', "_"));
                        match java.save_favicon(&filename) {
                            Ok(_) => println!("Favicon saved as {}", filename),
                            Err(e) => println!("Favicon not available: {}", e),
                        }
                    }
                    rust_mc_status::ServerData::Bedrock(bedrock) => {
                        println!("Name: {}", bedrock.motd);
                        println!("Version: {}", bedrock.version);
                        println!("Players: {}/{}", bedrock.online_players, bedrock.max_players);
                        println!("Favicon: Not supported in Bedrock");
                    }
                }
            }
            Err(e) => println!("Status: Offline or error ({})", e),
        }
    }

    Ok(())
}