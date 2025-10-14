use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::core::error_handler::{AppError, Result};
use std::net::{TcpListener, UdpSocket};
use std::time::Duration;

/// Port registry to track and prevent port conflicts
#[derive(Debug)]
pub struct PortRegistry {
    /// Maps port numbers to server IDs that are using them
    port_assignments: Arc<RwLock<HashMap<u16, Uuid>>>,
    /// Maps server IDs to their assigned ports
    server_ports: Arc<RwLock<HashMap<Uuid, Vec<u16>>>>,
}

impl PortRegistry {
    pub fn new() -> Self {
        Self {
            port_assignments: Arc::new(RwLock::new(HashMap::new())),
            server_ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a port is available and reserve it for a server
    pub async fn reserve_ports(&self, server_id: Uuid, ports: &[u16]) -> Result<()> {
        let mut port_assignments = self.port_assignments.write().await;
        let mut server_ports = self.server_ports.write().await;

        // Check if any of the ports are already in use
        for &port in ports {
            if port_assignments.contains_key(&port) {
                let existing_server = port_assignments.get(&port).unwrap();
                return Err(AppError::ValidationError {
                    message: format!("Port {} is already in use by server {}", port, existing_server),
                    field: "port".to_string(),
                    value: port.to_string(),
                    constraint: "must be available".to_string(),
                });
            }
        }

        // Check if ports are actually available on the system
        for &port in ports {
            if !self.is_port_available(port).await {
                return Err(AppError::ValidationError {
                    message: format!("Port {} is not available on the system", port),
                    field: "port".to_string(),
                    value: port.to_string(),
                    constraint: "must be available on system".to_string(),
                });
            }
        }

        // Reserve all ports
        for &port in ports {
            port_assignments.insert(port, server_id);
        }
        server_ports.insert(server_id, ports.to_vec());

        Ok(())
    }

    /// Release ports when a server is stopped or deleted
    pub async fn release_ports(&self, server_id: Uuid) -> Result<()> {
        let mut port_assignments = self.port_assignments.write().await;
        let mut server_ports = self.server_ports.write().await;

        if let Some(ports) = server_ports.remove(&server_id) {
            for port in ports {
                port_assignments.remove(&port);
            }
        }

        Ok(())
    }

    /// Check if a specific port is available on the system
    async fn is_port_available(&self, port: u16) -> bool {
        // Try to bind to the port to check if it's available
        let tcp_result = TcpListener::bind(format!("0.0.0.0:{}", port));
        let udp_result = UdpSocket::bind(format!("0.0.0.0:{}", port));

        tcp_result.is_ok() && udp_result.is_ok()
    }

    /// Get all ports assigned to a server
    pub async fn get_server_ports(&self, server_id: Uuid) -> Result<Vec<u16>> {
        let server_ports = self.server_ports.read().await;
        Ok(server_ports.get(&server_id).cloned().unwrap_or_default())
    }

    /// Get the server using a specific port
    pub async fn get_port_owner(&self, port: u16) -> Option<Uuid> {
        let port_assignments = self.port_assignments.read().await;
        port_assignments.get(&port).cloned()
    }

    /// Get all currently assigned ports
    pub async fn get_all_assigned_ports(&self) -> HashMap<u16, Uuid> {
        let port_assignments = self.port_assignments.read().await;
        port_assignments.clone()
    }

    /// Find available ports in a range
    pub async fn find_available_ports(&self, start_port: u16, count: u16) -> Result<Vec<u16>> {
        let mut available_ports = Vec::new();
        let mut current_port = start_port;

        while available_ports.len() < count as usize && current_port < u16::MAX {
            if self.is_port_available(current_port).await {
                // Check if it's not already assigned in our registry
                let port_assignments = self.port_assignments.read().await;
                if !port_assignments.contains_key(&current_port) {
                    available_ports.push(current_port);
                }
            }
            current_port += 1;
        }

        if available_ports.len() < count as usize {
            return Err(AppError::ValidationError {
                message: format!("Could not find {} available ports starting from {}", count, start_port),
                field: "ports".to_string(),
                value: format!("{} ports", count),
                constraint: "must be available".to_string(),
            });
        }

        Ok(available_ports)
    }

    /// Validate that a server's ports are still available
    pub async fn validate_server_ports(&self, server_id: Uuid) -> Result<()> {
        let server_ports = self.server_ports.read().await;
        
        if let Some(ports) = server_ports.get(&server_id) {
            for &port in ports {
                if !self.is_port_available(port).await {
                    return Err(AppError::ValidationError {
                        message: format!("Port {} is no longer available for server {}", port, server_id),
                        field: "port".to_string(),
                        value: port.to_string(),
                        constraint: "must be available".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for PortRegistry {
    fn default() -> Self {
        Self::new()
    }
}
