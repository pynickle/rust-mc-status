use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::McError;
use std::fs::File;
use std::io::Write;
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerStatus {
    pub online: bool,
    pub latency: f64,
    pub data: ServerData,
}

impl fmt::Debug for JavaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaStatus")
            .field("version", &self.version)
            .field("players", &self.players)
            .field("description", &self.description)
            .field("favicon", &self.favicon.as_ref().map(|_| "[Favicon data]"))
            .field("raw_data", &"[Value]")
            .finish()
    }
}

impl JavaStatus {
    pub fn save_favicon(&self, filename: &str) -> Result<(), McError> {
        if let Some(favicon) = &self.favicon {
            let data = favicon.split(',').nth(1).unwrap_or(favicon);
            let bytes = general_purpose::STANDARD.decode(data)
                .map_err(|e| McError::Base64Error(e))?;

            let mut file = File::create(filename)
                .map_err(|e| McError::IoError(e))?;

            file.write_all(&bytes)
                .map_err(|e| McError::IoError(e))?;

            Ok(())
        } else {
            Err(McError::InvalidResponse("No favicon available".to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ServerData {
    Java(JavaStatus),
    Bedrock(BedrockStatus),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JavaStatus {
    pub version: JavaVersion,
    pub players: JavaPlayers,
    pub description: String,
    #[serde(skip_serializing)]
    pub favicon: Option<String>,
    #[serde(skip)]
    pub raw_data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaVersion {
    pub name: String,
    pub protocol: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaPlayers {
    pub online: i64,
    pub max: i64,
    pub sample: Option<Vec<JavaPlayer>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaPlayer {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BedrockStatus {
    pub edition: String,
    pub motd: String,
    pub protocol_version: String,
    pub version: String,
    pub online_players: String,
    pub max_players: String,
    pub server_uid: String,
    pub motd2: String,
    pub game_mode: String,
    pub game_mode_numeric: String,
    pub port_ipv4: String,
    pub port_ipv6: String,
    pub raw_data: String,
}

impl fmt::Debug for BedrockStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BedrockStatus")
            .field("edition", &self.edition)
            .field("motd", &self.motd)
            .field("protocol_version", &self.protocol_version)
            .field("version", &self.version)
            .field("online_players", &self.online_players)
            .field("max_players", &self.max_players)
            .field("server_uid", &self.server_uid)
            .field("motd2", &self.motd2)
            .field("game_mode", &self.game_mode)
            .field("game_mode_numeric", &self.game_mode_numeric)
            .field("port_ipv4", &self.port_ipv4)
            .field("port_ipv6", &self.port_ipv6)
            .field("raw_data", &"[String]")
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub address: String,
    pub edition: ServerEdition,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ServerEdition {
    Java,
    Bedrock,
}

impl std::str::FromStr for ServerEdition {
    type Err = McError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "java" => Ok(ServerEdition::Java),
            "bedrock" => Ok(ServerEdition::Bedrock),
            _ => Err(McError::InvalidEdition(s.to_string())),
        }
    }
}