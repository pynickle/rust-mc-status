use crate::error::McError;
use crate::models::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

static DNS_CACHE: Lazy<DashMap<String, SocketAddr>> = Lazy::new(DashMap::new);

#[derive(Clone)]
pub struct McClient {
    timeout: Duration,
    max_parallel: usize,
}

impl Default for McClient {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_parallel: 10,
        }
    }
}

impl McClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_parallel(mut self, max_parallel: usize) -> Self {
        self.max_parallel = max_parallel;
        self
    }

    pub async fn ping(&self, address: &str, edition: ServerEdition) -> Result<ServerStatus, McError> {
        match edition {
            ServerEdition::Java => self.ping_java(address).await,
            ServerEdition::Bedrock => self.ping_bedrock(address).await,
        }
    }

    pub async fn ping_java(&self, address: &str) -> Result<ServerStatus, McError> {
        let start = SystemTime::now();
        let (host, port_str) = address.split_once(':').unwrap_or((address, "25565"));
        let port = port_str.parse::<u16>().map_err(|e| McError::InvalidPort(e.to_string()))?;

        let resolved = self.resolve_dns(host, port).await?;

        let mut stream = timeout(self.timeout, TcpStream::connect(resolved))
            .await
            .map_err(|_| McError::Timeout)?
            .map_err(|e| McError::ConnectionError(e.to_string()))?;

        stream.set_nodelay(true).map_err(|e| McError::IoError(e))?;

        // Handshake packet
        let mut handshake = Vec::with_capacity(64);
        write_var_int(&mut handshake, 0x00);
        write_var_int(&mut handshake, 47);
        write_string(&mut handshake, host);
        handshake.extend_from_slice(&port.to_be_bytes());
        write_var_int(&mut handshake, 1);

        let mut packet = Vec::with_capacity(handshake.len() + 5);
        write_var_int(&mut packet, handshake.len() as i32);
        packet.extend_from_slice(&handshake);

        timeout(self.timeout, stream.write_all(&packet))
            .await
            .map_err(|_| McError::Timeout)?
            .map_err(|e| McError::IoError(e))?;

        // Status request packet
        let mut status_request = Vec::with_capacity(5);
        write_var_int(&mut status_request, 0x00);

        let mut status_packet = Vec::with_capacity(status_request.len() + 5);
        write_var_int(&mut status_packet, status_request.len() as i32);
        status_packet.extend_from_slice(&status_request);

        timeout(self.timeout, stream.write_all(&status_packet))
            .await
            .map_err(|_| McError::Timeout)?
            .map_err(|e| McError::IoError(e))?;

        // Read response
        let mut response = Vec::with_capacity(1024);
        let mut buf = [0u8; 4096];

        loop {
            let read_result = timeout(self.timeout, stream.read(&mut buf)).await;

            match read_result {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    response.extend_from_slice(&buf[..n]);

                    if response.len() >= 5 {
                        let mut cursor = std::io::Cursor::new(&response);
                        if let Ok(packet_length) = read_var_int(&mut cursor) {
                            let total_length = cursor.position() as usize + packet_length as usize;
                            if response.len() >= total_length {
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => return Err(McError::IoError(e)),
                Err(_) => return Err(McError::Timeout),
            }
        }

        if response.is_empty() {
            return Err(McError::InvalidResponse("No response from server".to_string()));
        }

        let mut cursor = std::io::Cursor::new(&response);
        let packet_length = read_var_int(&mut cursor).map_err(|e| McError::InvalidResponse(e))?;

        let total_expected = cursor.position() as usize + packet_length as usize;
        if response.len() < total_expected {
            return Err(McError::InvalidResponse(format!("Incomplete packet: expected {}, got {}", total_expected, response.len())));
        }

        let packet_id = read_var_int(&mut cursor).map_err(|e| McError::InvalidResponse(e))?;
        if packet_id != 0x00 {
            return Err(McError::InvalidResponse(format!("Unexpected packet ID: {}", packet_id)));
        }

        let json_length = read_var_int(&mut cursor).map_err(|e| McError::InvalidResponse(e))?;
        if cursor.position() as usize + json_length as usize > response.len() {
            return Err(McError::InvalidResponse("JSON data truncated".to_string()));
        }

        let json_buf = &response[cursor.position() as usize..cursor.position() as usize + json_length as usize];
        let json_str = String::from_utf8(json_buf.to_vec()).map_err(|e| McError::Utf8Error(e))?;
        let json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| McError::JsonError(e))?;

        let latency = start.elapsed().map_err(|_| McError::InvalidResponse("Time error".to_string()))?.as_secs_f64() * 1000.0;

        // Parse JSON into structured data
        let version = JavaVersion {
            name: json["version"]["name"].as_str().unwrap_or("Unknown").to_string(),
            protocol: json["version"]["protocol"].as_i64().unwrap_or(0),
        };

        let players = JavaPlayers {
            online: json["players"]["online"].as_i64().unwrap_or(0),
            max: json["players"]["max"].as_i64().unwrap_or(0),
            sample: if let Some(sample) = json["players"]["sample"].as_array() {
                Some(sample.iter().filter_map(|p| {
                    Some(JavaPlayer {
                        name: p["name"].as_str()?.to_string(),
                        id: p["id"].as_str()?.to_string(),
                    })
                }).collect())
            } else {
                None
            },
        };

        let description = if let Some(desc) = json["description"].as_str() {
            desc.to_string()
        } else if let Some(text) = json["description"]["text"].as_str() {
            text.to_string()
        } else {
            "No description".to_string()
        };

        let favicon = json["favicon"].as_str().map(|s| s.to_string());

        Ok(ServerStatus {
            online: true,
            latency,
            data: ServerData::Java(JavaStatus {
                version,
                players,
                description,
                favicon,
                raw_data: json,
            }),
        })
    }

    pub async fn ping_bedrock(&self, address: &str) -> Result<ServerStatus, McError> {
        let start = SystemTime::now();
        let (host, port_str) = address.split_once(':').unwrap_or((address, "19132"));
        let port = port_str.parse::<u16>().map_err(|e| McError::InvalidPort(e.to_string()))?;

        let resolved = self.resolve_dns(host, port).await?;

        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| McError::IoError(e))?;

        let mut ping_packet = Vec::with_capacity(35);
        ping_packet.push(0x01);
        ping_packet.extend_from_slice(&(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64)
            .to_be_bytes());
        ping_packet.extend_from_slice(&[0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34, 0x56, 0x78]);
        ping_packet.extend_from_slice(&[0x00; 8]);

        timeout(self.timeout, socket.send_to(&ping_packet, resolved))
            .await
            .map_err(|_| McError::Timeout)?
            .map_err(|e| McError::IoError(e))?;

        let mut buf = [0u8; 1024];
        let (len, _) = timeout(self.timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| McError::Timeout)?
            .map_err(|e| McError::IoError(e))?;

        let latency = start.elapsed().map_err(|_| McError::InvalidResponse("Time error".to_string()))?.as_secs_f64() * 1000.0;

        if len < 35 {
            return Err(McError::InvalidResponse("Response too short".to_string()));
        }

        let pong_data = String::from_utf8_lossy(&buf[35..len]).to_string();
        let parts: Vec<&str> = pong_data.split(';').collect();

        if parts.len() < 6 {
            return Err(McError::InvalidResponse("Invalid Bedrock response".to_string()));
        }

        let status = BedrockStatus {
            edition: parts[0].to_string(),
            motd: parts[1].to_string(),
            protocol_version: parts[2].to_string(),
            version: parts[3].to_string(),
            online_players: parts[4].to_string(),
            max_players: parts[5].to_string(),
            server_uid: parts.get(6).map_or("", |s| *s).to_string(),
            motd2: parts.get(7).map_or("", |s| *s).to_string(),
            game_mode: parts.get(8).map_or("", |s| *s).to_string(),
            game_mode_numeric: parts.get(9).map_or("", |s| *s).to_string(),
            port_ipv4: parts.get(10).map_or("", |s| *s).to_string(),
            port_ipv6: parts.get(11).map_or("", |s| *s).to_string(),
            raw_data: pong_data,
        };

        Ok(ServerStatus {
            online: true,
            latency,
            data: ServerData::Bedrock(status),
        })
    }

    pub async fn ping_many(&self, servers: &[ServerInfo]) -> Vec<(ServerInfo, Result<ServerStatus, McError>)> {
        use futures::stream::StreamExt;
        use tokio::sync::Semaphore;

        let semaphore = std::sync::Arc::new(Semaphore::new(self.max_parallel));
        let client = self.clone();

        let futures = servers.iter().map(|server| {
            let server = server.clone();
            let semaphore = semaphore.clone();
            let client = client.clone();

            async move {
                let _permit = semaphore.acquire().await;
                let result = client.ping(&server.address, server.edition).await;
                (server, result)
            }
        });

        let mut results = Vec::new();
        let mut stream = futures::stream::iter(futures).buffer_unordered(self.max_parallel);

        while let Some(result) = stream.next().await {
            results.push(result);
        }

        results
    }

    async fn resolve_dns(&self, host: &str, port: u16) -> Result<SocketAddr, McError> {
        let cache_key = format!("{}:{}", host, port);

        if let Some(addr) = DNS_CACHE.get(&cache_key) {
            return Ok(*addr);
        }

        let addrs: Vec<SocketAddr> = format!("{}:{}", host, port)
            .to_socket_addrs()
            .map_err(|e| McError::DnsError(e.to_string()))?
            .collect();

        if addrs.is_empty() {
            return Err(McError::DnsError("No addresses resolved".to_string()));
        }

        // IPv4 for better compatibility
        let addr = addrs.iter()
            .find(|a| a.is_ipv4())
            .or_else(|| addrs.first())
            .copied()
            .unwrap();

        DNS_CACHE.insert(cache_key, addr);
        Ok(addr)
    }
}

// Helper functions
fn write_var_int(buffer: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut temp = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0x80;
        }
        buffer.push(temp);
        if value == 0 {
            break;
        }
    }
}

fn write_string(buffer: &mut Vec<u8>, s: &str) {
    write_var_int(buffer, s.len() as i32);
    buffer.extend_from_slice(s.as_bytes());
}

fn read_var_int(reader: &mut impl std::io::Read) -> Result<i32, String> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let mut byte = [0u8];
        reader.read_exact(&mut byte).map_err(|e| e.to_string())?;
        let value = byte[0] as i32;
        result |= (value & 0x7F) << shift;
        shift += 7;
        if shift > 35 {
            return Err("VarInt too big".to_string());
        }
        if (value & 0x80) == 0 {
            break;
        }
    }
    Ok(result)
}