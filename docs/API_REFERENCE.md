# Guardian Server Manager - API Reference

## Overview

The Guardian Server Manager provides a comprehensive REST API for managing Minecraft servers, mods, and system resources. This document describes all available endpoints, request/response formats, and usage examples.

## Base URL

```
http://localhost:8080/api
```

## Authentication

All API requests require authentication via API key or JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

## Response Format

All API responses follow a consistent format:

```json
{
  "success": true,
  "data": <response_data>,
  "message": "Optional message",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

Error responses:

```json
{
  "success": false,
  "error": "Error message",
  "code": "ERROR_CODE",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Server Management

### List All Servers

**GET** `/servers`

Returns a list of all servers.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "server-uuid",
      "name": "My Server",
      "status": "running",
      "version": "1.20.1",
      "maxPlayers": 20,
      "tps": 20.0,
      "tickP95": 45.2,
      "heapMb": 2048,
      "playersOnline": 5,
      "gpuQueueMs": 0.0,
      "lastSnapshotAt": "2024-01-01T00:00:00Z",
      "blueGreen": {
        "active": "blue",
        "candidateHealthy": true
      },
      "createdAt": "2024-01-01T00:00:00Z",
      "updatedAt": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### Get Server Details

**GET** `/servers/{id}`

Returns detailed information about a specific server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-uuid",
    "name": "My Server",
    "description": "A Minecraft server",
    "status": "running",
    "version": "1.20.1",
    "modpack": "FTB Skies",
    "maxPlayers": 20,
    "motd": "Welcome to my server!",
    "difficulty": "normal",
    "gamemode": "survival",
    "pvp": true,
    "onlineMode": true,
    "whitelist": false,
    "enableCommandBlock": false,
    "viewDistance": 10,
    "simulationDistance": 10,
    "jvm": {
      "memory": 4096,
      "flags": ["-XX:+UseG1GC"],
      "gcType": "G1GC"
    },
    "gpu": {
      "enabled": true,
      "queueSize": 100,
      "maxWorkers": 4
    },
    "createdAt": "2024-01-01T00:00:00Z",
    "updatedAt": "2024-01-01T00:00:00Z"
  }
}
```

### Create Server

**POST** `/servers`

Creates a new server.

**Request Body:**
```json
{
  "name": "My New Server",
  "description": "A new Minecraft server",
  "version": "1.20.1",
  "modpack": "FTB Skies",
  "maxPlayers": 20,
  "motd": "Welcome!",
  "difficulty": "normal",
  "gamemode": "survival",
  "pvp": true,
  "onlineMode": true,
  "whitelist": false,
  "enableCommandBlock": false,
  "viewDistance": 10,
  "simulationDistance": 10,
  "jvm": {
    "memory": 4096,
    "flags": ["-XX:+UseG1GC"],
    "gcType": "G1GC"
  },
  "gpu": {
    "enabled": true,
    "queueSize": 100,
    "maxWorkers": 4
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "new-server-uuid",
    "name": "My New Server",
    "status": "stopped",
    "createdAt": "2024-01-01T00:00:00Z"
  }
}
```

### Update Server

**PUT** `/servers/{id}`

Updates server configuration.

**Parameters:**
- `id` (string): Server UUID

**Request Body:** Same as create server

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-uuid",
    "updatedAt": "2024-01-01T00:00:00Z"
  }
}
```

### Delete Server

**DELETE** `/servers/{id}`

Deletes a server and all associated data.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "message": "Server deleted successfully"
}
```

### Server Actions

#### Start Server

**POST** `/servers/{id}/start`

Starts a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "message": "Server starting"
}
```

#### Stop Server

**POST** `/servers/{id}/stop`

Stops a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "message": "Server stopping"
}
```

#### Restart Server

**POST** `/servers/{id}/restart`

Restarts a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "message": "Server restarting"
}
```

## Player Management

### Get Players

**GET** `/servers/{id}/players`

Returns list of players on a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "uuid": "player-uuid",
      "name": "PlayerName",
      "online": true,
      "lastSeen": "2024-01-01T00:00:00Z",
      "playtime": 3600,
      "ip": "192.168.1.100"
    }
  ]
}
```

### Kick Player

**POST** `/servers/{id}/players/{uuid}/kick`

Kicks a player from the server.

**Parameters:**
- `id` (string): Server UUID
- `uuid` (string): Player UUID

**Request Body:**
```json
{
  "reason": "You have been kicked"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Player kicked"
}
```

### Ban Player

**POST** `/servers/{id}/players/{uuid}/ban`

Bans a player from the server.

**Parameters:**
- `id` (string): Server UUID
- `uuid` (string): Player UUID

**Request Body:**
```json
{
  "reason": "You have been banned"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Player banned"
}
```

## Console Management

### Send Command

**POST** `/servers/{id}/console/command`

Sends a command to the server console.

**Parameters:**
- `id` (string): Server UUID

**Request Body:**
```json
{
  "command": "say Hello World"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Command sent"
}
```

### Get Console Output

**GET** `/servers/{id}/console/output`

Returns recent console output.

**Parameters:**
- `id` (string): Server UUID
- `lines` (query, optional): Number of lines to return (default: 100)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "level": "INFO",
      "message": "Server started"
    }
  ]
}
```

## Mod Management

### Search Mods

**GET** `/mods/search`

Searches for mods on CurseForge and Modrinth.

**Query Parameters:**
- `query` (string): Search query
- `minecraft_version` (string, optional): Minecraft version filter
- `loader` (string, optional): Mod loader filter (forge, fabric, quilt)
- `category` (string, optional): Category filter
- `source` (string, optional): Source filter (curseforge, modrinth)
- `limit` (number, optional): Number of results (default: 20)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "mod-id",
      "name": "Mod Name",
      "description": "Mod description",
      "version": "1.0.0",
      "minecraftVersion": "1.20.1",
      "loader": "fabric",
      "source": "modrinth",
      "downloadUrl": "https://...",
      "fileSize": 1024000,
      "dependencies": [
        {
          "modId": "fabric-api",
          "version": ">=0.80.0",
          "required": true
        }
      ]
    }
  ]
}
```

### Get Mod Details

**GET** `/mods/{id}`

Returns detailed information about a mod.

**Parameters:**
- `id` (string): Mod ID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "mod-id",
    "name": "Mod Name",
    "description": "Detailed mod description",
    "author": "Author Name",
    "version": "1.0.0",
    "minecraftVersion": "1.20.1",
    "loader": "fabric",
    "source": "modrinth",
    "downloadUrl": "https://...",
    "fileSize": 1024000,
    "dependencies": [...],
    "screenshots": ["https://..."],
    "links": {
      "homepage": "https://...",
      "issues": "https://...",
      "source": "https://..."
    }
  }
}
```

### Install Mod

**POST** `/servers/{id}/mods/install`

Installs a mod on a server.

**Parameters:**
- `id` (string): Server UUID

**Request Body:**
```json
{
  "modId": "mod-id",
  "version": "1.0.0",
  "provider": "modrinth",
  "filePath": "/path/to/mod.jar",
  "serverId": "server-uuid"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Mod installed successfully"
}
```

### Remove Mod

**POST** `/servers/{id}/mods/remove`

Removes a mod from a server.

**Parameters:**
- `id` (string): Server UUID

**Request Body:**
```json
{
  "modId": "mod-id",
  "filePath": "/path/to/mod.jar",
  "serverId": "server-uuid"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Mod removed successfully"
}
```

### Update Mod

**POST** `/servers/{id}/mods/update`

Updates a mod to a new version.

**Parameters:**
- `id` (string): Server UUID

**Request Body:**
```json
{
  "modId": "mod-id",
  "fromVersion": "1.0.0",
  "toVersion": "1.1.0",
  "provider": "modrinth",
  "filePath": "/path/to/mod.jar",
  "serverId": "server-uuid"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Mod updated successfully"
}
```

## Performance Monitoring

### Get Server Metrics

**GET** `/performance/{id}/metrics`

Returns performance metrics for a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "tps": 20.0,
      "tickMs": 50.0,
      "memoryUsageMb": 2048,
      "memoryMaxMb": 4096,
      "memoryUtilizationPercent": 50.0,
      "cpuUsagePercent": 25.0,
      "diskReadBytesPerSec": 1048576,
      "diskWriteBytesPerSec": 524288,
      "networkInBytesPerSec": 1048576,
      "networkOutBytesPerSec": 524288,
      "diskUsageMb": 10240
    }
  ]
}
```

### Get Performance Summary

**GET** `/performance/{id}/summary`

Returns performance summary for a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": {
    "serverId": "server-uuid",
    "avgTps": 19.8,
    "maxTps": 20.0,
    "minTps": 18.5,
    "avgTickMs": 50.5,
    "maxTickMs": 100.0,
    "minTickMs": 45.0,
    "avgMemoryUtilizationPercent": 55.0,
    "maxMemoryUtilizationPercent": 80.0,
    "avgCpuUsagePercent": 30.0,
    "maxCpuUsagePercent": 60.0,
    "totalDiskReadMb": 1024,
    "totalDiskWriteMb": 512,
    "totalNetworkInMb": 2048,
    "totalNetworkOutMb": 1024,
    "currentDiskUsageMb": 10240
  }
}
```

## Compatibility Analysis

### Get Compatibility Issues

**GET** `/compatibility/{id}/issues`

Returns compatibility issues for a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "issue-id",
      "modId": "mod-id",
      "modName": "Mod Name",
      "issueType": "conflict",
      "severity": "high",
      "description": "Mod conflicts with another mod",
      "conflictingMods": ["other-mod-id"],
      "recommendations": [
        {
          "action": "remove",
          "modId": "other-mod-id",
          "priority": "high",
          "description": "Remove conflicting mod"
        }
      ]
    }
  ]
}
```

### Get Risk Analysis

**GET** `/compatibility/{id}/risk-analysis`

Returns risk analysis for all mods on a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "modId": "mod-id",
      "overallScore": 0.3,
      "riskLevel": "low",
      "incompatibilityScore": 0.0,
      "dependencyScore": 0.2,
      "performanceScore": 0.1,
      "stabilityScore": 0.0,
      "recommendations": ["Mod appears stable"]
    }
  ]
}
```

### Apply Compatibility Fix

**POST** `/compatibility/{id}/fixes/{fixId}/apply`

Applies a compatibility fix.

**Parameters:**
- `id` (string): Server UUID
- `fixId` (string): Fix ID

**Response:**
```json
{
  "success": true,
  "message": "Fix applied successfully"
}
```

## Backup Management

### Create Backup

**POST** `/servers/{id}/backups`

Creates a backup of a server.

**Parameters:**
- `id` (string): Server UUID

**Request Body:**
```json
{
  "name": "Backup Name",
  "description": "Backup description",
  "includeWorld": true,
  "includeMods": true,
  "includeConfig": true,
  "compression": "zip"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "backup-id",
    "name": "Backup Name",
    "size": 104857600,
    "createdAt": "2024-01-01T00:00:00Z"
  }
}
```

### List Backups

**GET** `/servers/{id}/backups`

Returns list of backups for a server.

**Parameters:**
- `id` (string): Server UUID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "backup-id",
      "name": "Backup Name",
      "description": "Backup description",
      "size": 104857600,
      "createdAt": "2024-01-01T00:00:00Z",
      "includes": {
        "world": true,
        "mods": true,
        "config": true
      }
    }
  ]
}
```

### Restore Backup

**POST** `/servers/{id}/backups/{backupId}/restore`

Restores a server from a backup.

**Parameters:**
- `id` (string): Server UUID
- `backupId` (string): Backup ID

**Request Body:**
```json
{
  "includeWorld": true,
  "includeMods": true,
  "includeConfig": true
}
```

**Response:**
```json
{
  "success": true,
  "message": "Backup restored successfully"
}
```

### Delete Backup

**DELETE** `/servers/{id}/backups/{backupId}`

Deletes a backup.

**Parameters:**
- `id` (string): Server UUID
- `backupId` (string): Backup ID

**Response:**
```json
{
  "success": true,
  "message": "Backup deleted successfully"
}
```

## GPU Management

### Get GPU Status

**GET** `/gpu/status`

Returns GPU worker status and capabilities.

**Response:**
```json
{
  "success": true,
  "data": {
    "available": true,
    "provider": "webgpu",
    "deviceName": "NVIDIA GeForce RTX 4090",
    "memoryTotal": 24576,
    "memoryUsed": 1024,
    "queueSize": 0,
    "activeJobs": 0,
    "maxWorkers": 4
  }
}
```

### Submit GPU Job

**POST** `/gpu/jobs`

Submits a job to the GPU worker.

**Request Body:**
```json
{
  "type": "chunk_generation",
  "serverId": "server-uuid",
  "parameters": {
    "x": 0,
    "z": 0,
    "seed": 12345,
    "dimension": "overworld"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "jobId": "job-uuid",
    "status": "queued",
    "estimatedDuration": 5000
  }
}
```

### Get GPU Job Status

**GET** `/gpu/jobs/{jobId}`

Returns status of a GPU job.

**Parameters:**
- `jobId` (string): Job UUID

**Response:**
```json
{
  "success": true,
  "data": {
    "jobId": "job-uuid",
    "status": "completed",
    "progress": 100,
    "result": {
      "chunkData": "base64-encoded-data",
      "metadata": {...}
    },
    "duration": 4500,
    "createdAt": "2024-01-01T00:00:00Z",
    "completedAt": "2024-01-01T00:00:05Z"
  }
}
```

## WebSocket Events

The API also provides WebSocket endpoints for real-time updates:

### Connect to WebSocket

**WebSocket** `/ws`

Connects to the WebSocket for real-time updates.

### Event Types

#### Server Status Update
```json
{
  "type": "server_status",
  "serverId": "server-uuid",
  "status": "running",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Performance Metrics
```json
{
  "type": "metrics",
  "serverId": "server-uuid",
  "data": {
    "tps": 20.0,
    "tickMs": 50.0,
    "memoryUsage": 2048,
    "cpuUsage": 25.0
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Console Output
```json
{
  "type": "console_output",
  "serverId": "server-uuid",
  "data": {
    "level": "INFO",
    "message": "Server started"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `SERVER_NOT_FOUND` | Server with specified ID not found |
| `MOD_NOT_FOUND` | Mod with specified ID not found |
| `BACKUP_NOT_FOUND` | Backup with specified ID not found |
| `INVALID_REQUEST` | Request body is invalid |
| `UNAUTHORIZED` | Authentication required |
| `FORBIDDEN` | Insufficient permissions |
| `SERVER_BUSY` | Server is currently busy |
| `GPU_UNAVAILABLE` | GPU worker is not available |
| `BACKUP_FAILED` | Backup operation failed |
| `RESTORE_FAILED` | Restore operation failed |

## Rate Limiting

API requests are rate limited to prevent abuse:

- **General API**: 1000 requests per hour per IP
- **Server Actions**: 10 requests per minute per server
- **Mod Operations**: 50 requests per minute per server
- **Backup Operations**: 5 requests per hour per server

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Pagination

List endpoints support pagination:

**Query Parameters:**
- `page` (number): Page number (default: 1)
- `limit` (number): Items per page (default: 20)

**Response Headers:**
```
X-Total-Count: 100
X-Page-Count: 5
X-Current-Page: 1
X-Per-Page: 20
```

## Examples

### Complete Server Setup

```bash
# 1. Create server
curl -X POST http://localhost:8080/api/servers \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Server",
    "version": "1.20.1",
    "maxPlayers": 20
  }'

# 2. Start server
curl -X POST http://localhost:8080/api/servers/{id}/start \
  -H "Authorization: Bearer <token>"

# 3. Install mod
curl -X POST http://localhost:8080/api/servers/{id}/mods/install \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "modId": "fabric-api",
    "version": "0.80.0",
    "provider": "modrinth"
  }'

# 4. Create backup
curl -X POST http://localhost:8080/api/servers/{id}/backups \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Initial Setup",
    "includeWorld": true,
    "includeMods": true
  }'
```

### Real-time Monitoring

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'server_status':
      updateServerStatus(data.serverId, data.status);
      break;
    case 'metrics':
      updatePerformanceChart(data.serverId, data.data);
      break;
    case 'console_output':
      appendConsoleOutput(data.serverId, data.data);
      break;
  }
};
```

This API reference provides comprehensive documentation for all available endpoints and their usage. For additional examples and advanced usage patterns, refer to the main documentation.
