use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};
use tracing::{debug, error, info, warn};

/// RCON client for Minecraft server communication
#[derive(Debug)]
pub struct RconClient {
    host: String,
    port: u16,
    password: String,
    timeout: Duration,
}

/// RCON packet structure
#[derive(Debug)]
struct RconPacket {
    length: i32,
    request_id: i32,
    packet_type: i32,
    body: String,
}

impl RconPacket {
    fn new(request_id: i32, packet_type: i32, body: String) -> Self {
        let body_bytes = body.as_bytes();
        let length = 4 + 4 + body_bytes.len() + 1; // request_id + packet_type + body + null terminator
        
        Self {
            length: length as i32,
            request_id,
            packet_type,
            body,
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Length
        data.extend_from_slice(&self.length.to_le_bytes());
        
        // Request ID
        data.extend_from_slice(&self.request_id.to_le_bytes());
        
        // Packet type
        data.extend_from_slice(&self.packet_type.to_le_bytes());
        
        // Body
        data.extend_from_slice(self.body.as_bytes());
        
        // Null terminator
        data.push(0);
        
        data
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(anyhow!("Packet too short"));
        }

        let length = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let request_id = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let packet_type = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        
        let body = if data.len() > 12 {
            String::from_utf8_lossy(&data[12..data.len()-1]).to_string() // Remove null terminator
        } else {
            String::new()
        };

        Ok(Self {
            length,
            request_id,
            packet_type,
            body,
        })
    }
}

/// RCON packet types
const RCON_AUTH: i32 = 3;
const RCON_AUTH_RESPONSE: i32 = 2;
const RCON_COMMAND: i32 = 2;
const RCON_RESPONSE: i32 = 0;

impl RconClient {
    /// Create a new RCON client
    pub fn new(host: String, port: u16, password: String) -> Self {
        Self {
            host,
            port,
            password,
            timeout: Duration::from_secs(10),
        }
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Connect to the RCON server
    fn connect(&self) -> Result<TcpStream> {
        let addr = format!("{}:{}", self.host, self.port);
        debug!("Connecting to RCON server: {}", addr);
        
        let stream = TcpStream::connect_timeout(
            &addr.parse()?,
            self.timeout
        )?;
        
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        
        Ok(stream)
    }

    /// Send a packet and receive a response
    fn send_packet(&self, stream: &mut TcpStream, packet: RconPacket) -> Result<RconPacket> {
        let data = packet.serialize();
        debug!("Sending RCON packet: {:?}", packet);
        
        stream.write_all(&data)?;
        stream.flush()?;

        // Read response
        let mut response_data = Vec::new();
        let mut buffer = [0u8; 4096];
        
        loop {
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            response_data.extend_from_slice(&buffer[..bytes_read]);
            
            // Check if we have a complete packet
            if response_data.len() >= 4 {
                let length = i32::from_le_bytes([
                    response_data[0], response_data[1], response_data[2], response_data[3]
                ]) as usize;
                
                if response_data.len() >= length + 4 {
                    break;
                }
            }
        }

        let response = RconPacket::deserialize(&response_data)?;
        debug!("Received RCON response: {:?}", response);
        
        Ok(response)
    }

    /// Authenticate with the RCON server
    fn authenticate(&self, stream: &mut TcpStream) -> Result<()> {
        let auth_packet = RconPacket::new(1, RCON_AUTH, self.password.clone());
        let response = self.send_packet(stream, auth_packet)?;
        
        if response.packet_type == RCON_AUTH_RESPONSE && response.request_id == 1 {
            info!("RCON authentication successful");
            Ok(())
        } else {
            Err(anyhow!("RCON authentication failed"))
        }
    }

    /// Send a command to the server
    pub fn send_command(&self, command: &str) -> Result<String> {
        let mut stream = self.connect()?;
        
        // Authenticate
        self.authenticate(&mut stream)?;
        
        // Send command
        let command_packet = RconPacket::new(2, RCON_COMMAND, command.to_string());
        let response = self.send_packet(&mut stream, command_packet)?;
        
        if response.packet_type == RCON_RESPONSE {
            Ok(response.body)
        } else {
            Err(anyhow!("Invalid response packet type"))
        }
    }

    /// Check if RCON is available
    pub fn is_available(&self) -> bool {
        match self.connect() {
            Ok(_) => true,
            Err(e) => {
                debug!("RCON not available: {}", e);
                false
            }
        }
    }

    /// Test the connection and authentication
    pub fn test_connection(&self) -> Result<()> {
        let mut stream = self.connect()?;
        self.authenticate(&mut stream)?;
        
        // Send a simple command to test
        let test_packet = RconPacket::new(3, RCON_COMMAND, "list".to_string());
        let response = self.send_packet(&mut stream, test_packet)?;
        
        if response.packet_type == RCON_RESPONSE {
            info!("RCON connection test successful");
            Ok(())
        } else {
            Err(anyhow!("RCON connection test failed"))
        }
    }

    /// Get server information
    pub fn get_server_info(&self) -> Result<ServerInfo> {
        let list_response = self.send_command("list")?;
        let tps_response = self.send_command("tps")?;
        
        let players_online = self.parse_player_count(&list_response)?;
        let tps = self.parse_tps(&tps_response)?;
        
        Ok(ServerInfo {
            players_online,
            tps,
            list_response,
            tps_response,
        })
    }

    /// Parse player count from list command response
    fn parse_player_count(&self, response: &str) -> Result<u32> {
        // Parse response like "There are 5 of a max of 20 players online: Player1, Player2, ..."
        if let Some(start) = response.find("There are ") {
            let start = start + 11; // Length of "There are "
            if let Some(end) = response[start..].find(" of a max of") {
                let count_str = &response[start..start + end];
                return Ok(count_str.parse()?);
            }
        }
        Ok(0)
    }

    /// Parse TPS from tps command response
    fn parse_tps(&self, response: &str) -> Result<f64> {
        // Parse response like "TPS: 20.0 (1m, 5m, 15m)"
        if let Some(start) = response.find("TPS: ") {
            let start = start + 5; // Length of "TPS: "
            if let Some(end) = response[start..].find(" (") {
                let tps_str = &response[start..start + end];
                return Ok(tps_str.parse()?);
            }
        }
        Ok(20.0) // Default TPS
    }

    /// Get player list
    pub fn get_players(&self) -> Result<Vec<Player>> {
        let response = self.send_command("list")?;
        self.parse_players(&response)
    }

    /// Parse player list from list command response
    fn parse_players(&self, response: &str) -> Result<Vec<Player>> {
        let mut players = Vec::new();
        
        // Parse response like "There are 5 of a max of 20 players online: Player1, Player2, Player3, Player4, Player5"
        if let Some(players_part) = response.split(": ").nth(1) {
            for player_name in players_part.split(", ") {
                let name = player_name.trim();
                if !name.is_empty() {
                    players.push(Player {
                        uuid: uuid::Uuid::new_v4().to_string(), // TODO: Get actual UUID
                        name: name.to_string(),
                        online: true,
                        last_seen: Some(chrono::Utc::now()),
                        playtime: None,
                        ping: None,
                        dimension: None,
                        x: None,
                        y: None,
                        z: None,
                    });
                }
            }
        }
        
        Ok(players)
    }

    /// Kick a player
    pub fn kick_player(&self, player_name: &str, reason: Option<&str>) -> Result<String> {
        let command = if let Some(reason) = reason {
            format!("kick {} {}", player_name, reason)
        } else {
            format!("kick {}", player_name)
        };
        
        self.send_command(&command)
    }

    /// Ban a player
    pub fn ban_player(&self, player_name: &str, reason: Option<&str>) -> Result<String> {
        let command = if let Some(reason) = reason {
            format!("ban {} {}", player_name, reason)
        } else {
            format!("ban {}", player_name)
        };
        
        self.send_command(&command)
    }

    /// Send a message to a player
    pub fn message_player(&self, player_name: &str, message: &str) -> Result<String> {
        let command = format!("msg {} {}", player_name, message);
        self.send_command(&command)
    }

    /// Teleport a player
    pub fn teleport_player(&self, player_name: &str, x: f64, y: f64, z: f64) -> Result<String> {
        let command = format!("tp {} {} {} {}", player_name, x, y, z);
        self.send_command(&command)
    }

    /// Give an item to a player
    pub fn give_item(&self, player_name: &str, item: &str, count: Option<u32>) -> Result<String> {
        let command = if let Some(count) = count {
            format!("give {} {} {}", player_name, item, count)
        } else {
            format!("give {} {}", player_name, item)
        };
        
        self.send_command(&command)
    }

    /// Set the time
    pub fn set_time(&self, time: &str) -> Result<String> {
        let command = format!("time set {}", time);
        self.send_command(&command)
    }

    /// Set the weather
    pub fn set_weather(&self, weather: &str) -> Result<String> {
        let command = format!("weather {}", weather);
        self.send_command(&command)
    }

    /// Save the world
    pub fn save_world(&self) -> Result<String> {
        self.send_command("save-all")
    }

    /// Stop the server
    pub fn stop_server(&self) -> Result<String> {
        self.send_command("stop")
    }
}

/// Server information from RCON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub players_online: u32,
    pub tps: f64,
    pub list_response: String,
    pub tps_response: String,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub playtime: Option<u64>,
    pub ping: Option<u32>,
    pub dimension: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rcon_packet_serialization() {
        let packet = RconPacket::new(1, RCON_AUTH, "password".to_string());
        let data = packet.serialize();
        
        // Should have length, request_id, packet_type, body, and null terminator
        assert!(data.len() > 12);
        assert_eq!(data[data.len()-1], 0); // Null terminator
    }

    #[test]
    fn test_rcon_packet_deserialization() {
        let original = RconPacket::new(1, RCON_AUTH, "password".to_string());
        let data = original.serialize();
        let deserialized = RconPacket::deserialize(&data).unwrap();
        
        assert_eq!(original.request_id, deserialized.request_id);
        assert_eq!(original.packet_type, deserialized.packet_type);
        assert_eq!(original.body, deserialized.body);
    }

    #[test]
    fn test_parse_player_count() {
        let client = RconClient::new("localhost".to_string(), 25575, "password".to_string());
        
        let response = "There are 5 of a max of 20 players online: Player1, Player2, Player3, Player4, Player5";
        let count = client.parse_player_count(response).unwrap();
        assert_eq!(count, 5);
        
        let empty_response = "There are 0 of a max of 20 players online:";
        let count = client.parse_player_count(empty_response).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_tps() {
        let client = RconClient::new("localhost".to_string(), 25575, "password".to_string());
        
        let response = "TPS: 20.0 (1m, 5m, 15m)";
        let tps = client.parse_tps(response).unwrap();
        assert_eq!(tps, 20.0);
        
        let response = "TPS: 15.5 (1m, 5m, 15m)";
        let tps = client.parse_tps(response).unwrap();
        assert_eq!(tps, 15.5);
    }
}
