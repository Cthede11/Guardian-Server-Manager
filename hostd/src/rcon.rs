use anyhow::Result;
use std::net::TcpStream;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub dimension: String,
    pub last_seen: String,
    pub online: bool,
    pub playtime: u64,
    pub ping: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub player_count: u32,
    pub max_players: u32,
    pub tps: f64,
    pub uptime: String,
}

/// RCON client for Minecraft servers
pub struct RconClient {
    host: String,
    port: u16,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RconPacket {
    length: i32,
    request_id: i32,
    packet_type: i32,
    payload: String,
    padding: u8,
}

impl RconClient {
    pub fn new(host: String, port: u16, password: String) -> Self {
        Self { host, port, password }
    }
    
    pub fn is_available(&self) -> bool {
        let addr = format!("{}:{}", self.host, self.port);
        TcpStream::connect(addr).is_ok()
    }
    
    pub fn send_command(&self, command: &str) -> Result<String> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(addr)?;
        
        // Send authentication packet
        let auth_packet = self.create_packet(0, 3, &self.password);
        self.send_packet(&mut stream, &auth_packet)?;
        
        // Read authentication response
        let auth_response = self.read_packet(&mut stream)?;
        if auth_response.request_id == -1 {
            return Err(anyhow::anyhow!("Authentication failed"));
        }
        
        // Send command packet
        let command_packet = self.create_packet(auth_response.request_id, 2, command);
        self.send_packet(&mut stream, &command_packet)?;
        
        // Read command response
        let response = self.read_packet(&mut stream)?;
        
        Ok(response.payload)
    }
    
    pub fn get_players(&self) -> Result<Vec<Player>> {
        let response = self.send_command("list")?;
        self.parse_player_list(&response)
    }
    
    /// Parse the player list from the server response
    fn parse_player_list(&self, response: &str) -> Result<Vec<Player>> {
        let mut players = Vec::new();
        
        // The response format is typically: "There are X of a max of Y players online: player1, player2, ..."
        if response.contains("There are") && response.contains("players online:") {
            // Extract the player list part
            if let Some(colon_pos) = response.find("players online:") {
                let player_list = &response[colon_pos + 15..].trim();
                
                if !player_list.is_empty() && *player_list != "There are 0 of a max of" {
                    // Split by comma and parse each player
                    for player_name in player_list.split(',') {
                        let player_name = player_name.trim();
                        if !player_name.is_empty() {
                            players.push(Player {
                                uuid: Uuid::new_v4().to_string(), // We don't have UUID from list command
                                name: player_name.to_string(),
                                dimension: "overworld".to_string(), // Default dimension
                                last_seen: Utc::now().to_rfc3339(),
                                online: true,
                                playtime: 0, // Not available from list command
                                ping: 0, // Not available from list command
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(players)
    }
    
    /// Get detailed player information (requires additional commands)
    pub fn get_player_info(&self, player_name: &str) -> Result<Option<Player>> {
        // Try to get player data using the data command
        let response = self.send_command(&format!("data get entity {} Pos", player_name))?;
        
        if response.contains("No entity was found") {
            return Ok(None);
        }
        
        // Parse position data (format: "player has the following entity data: [x, y, z]")
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        
        if let Some(bracket_start) = response.find('[') {
            if let Some(bracket_end) = response.find(']') {
                let coords = &response[bracket_start + 1..bracket_end];
                let parts: Vec<&str> = coords.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 {
                    x = parts[0].parse().unwrap_or(0.0);
                    y = parts[1].parse().unwrap_or(0.0);
                    z = parts[2].parse().unwrap_or(0.0);
                }
            }
        }
        
        // Get dimension
        let dimension_response = self.send_command(&format!("data get entity {} Dimension", player_name))?;
        let dimension = if dimension_response.contains("minecraft:overworld") {
            "overworld".to_string()
        } else if dimension_response.contains("minecraft:nether") {
            "nether".to_string()
        } else if dimension_response.contains("minecraft:end") {
            "end".to_string()
        } else {
            "overworld".to_string()
        };
        
        Ok(Some(Player {
            uuid: Uuid::new_v4().to_string(), // We don't have UUID from these commands
            name: player_name.to_string(),
            dimension,
            last_seen: Utc::now().to_rfc3339(),
            online: true,
            playtime: 0, // Not easily available
            ping: 0, // Not easily available
            x,
            y,
            z,
        }))
    }
    
    /// Send a command and get the response
    pub fn execute_command(&self, command: &str) -> Result<String> {
        self.send_command(command)
    }
    
    /// Check if the server is responding
    pub fn ping(&self) -> Result<bool> {
        match self.send_command("list") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Get server information
    pub fn get_server_info(&self) -> Result<ServerInfo> {
        let list_response = self.send_command("list")?;
        let tps_response = self.send_command("tps")?;
        
        // Parse player count from list response
        let mut player_count = 0;
        let mut max_players = 0;
        
        if let Some(are_pos) = list_response.find("There are") {
            if let Some(of_pos) = list_response.find("of a max of") {
                if let Some(players_pos) = list_response.find("players online") {
                    let count_str = &list_response[are_pos + 10..of_pos].trim();
                    let max_str = &list_response[of_pos + 11..players_pos].trim();
                    
                    player_count = count_str.parse().unwrap_or(0);
                    max_players = max_str.parse().unwrap_or(0);
                }
            }
        }
        
        // Parse TPS from tps response (format varies by server)
        let mut tps = 20.0;
        if tps_response.contains("TPS") {
            // Try to extract TPS value
            for word in tps_response.split_whitespace() {
                if let Ok(parsed_tps) = word.parse::<f64>() {
                    if parsed_tps > 0.0 && parsed_tps <= 20.0 {
                        tps = parsed_tps;
                        break;
                    }
                }
            }
        }
        
        Ok(ServerInfo {
            player_count,
            max_players,
            tps,
            uptime: "unknown".to_string(), // Not easily available via RCON
        })
    }
    
    fn create_packet(&self, request_id: i32, packet_type: i32, payload: &str) -> RconPacket {
        RconPacket {
            length: (payload.len() + 10) as i32,
            request_id,
            packet_type,
            payload: payload.to_string(),
            padding: 0,
        }
    }
    
    fn send_packet(&self, stream: &mut TcpStream, packet: &RconPacket) -> Result<()> {
        let mut data = Vec::new();
        data.extend_from_slice(&packet.length.to_le_bytes());
        data.extend_from_slice(&packet.request_id.to_le_bytes());
        data.extend_from_slice(&packet.packet_type.to_le_bytes());
        data.extend_from_slice(packet.payload.as_bytes());
        data.push(packet.padding);
        data.push(0);
        
        stream.write_all(&data)?;
            Ok(())
    }
    
    fn read_packet(&self, stream: &mut TcpStream) -> Result<RconPacket> {
        let mut length_buf = [0u8; 4];
        stream.read_exact(&mut length_buf)?;
        let length = i32::from_le_bytes(length_buf);
        
        let mut request_id_buf = [0u8; 4];
        stream.read_exact(&mut request_id_buf)?;
        let request_id = i32::from_le_bytes(request_id_buf);
        
        let mut packet_type_buf = [0u8; 4];
        stream.read_exact(&mut packet_type_buf)?;
        let packet_type = i32::from_le_bytes(packet_type_buf);
        
        let payload_length = length - 10;
        let mut payload_buf = vec![0u8; payload_length as usize];
        stream.read_exact(&mut payload_buf)?;
        let payload = String::from_utf8(payload_buf)?;
        
        let mut padding_buf = [0u8; 2];
        stream.read_exact(&mut padding_buf)?;
        
        Ok(RconPacket {
            length,
            request_id,
            packet_type,
            payload,
            padding: padding_buf[0],
        })
    }
}