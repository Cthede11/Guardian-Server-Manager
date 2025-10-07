# Guardian Server Manager - API Reference

## Overview

The Guardian Server Manager provides a comprehensive REST API for managing Minecraft servers, mods, modpacks, and system resources. All API endpoints return JSON responses and follow RESTful conventions.

## Base URL

- **Development**: `http://127.0.0.1:52100`
- **Production**: Configured via environment variables

## Authentication

Currently, the API operates in localhost-only mode for security. Future versions will support JWT-based authentication.

## Response Format

All API responses follow this standard format:

```json
{
  "success": boolean,
  "data": any | null,
  "error": string | null,
  "timestamp": string
}
```

## Error Codes

- `200` - Success
- `400` - Bad Request (validation error)
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict
- `429` - Too Many Requests (rate limited)
- `500` - Internal Server Error

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Search endpoints**: 60 requests per minute
- **Download endpoints**: 30 requests per minute
- **Server management**: 10 requests per minute
- **Authentication**: 5 requests per minute

## Endpoints

### Health & System

#### GET /api/health

Get system health status with per-component health checks.

**Response:**
```json
{
  "success": true,
  "data": {
    "overall_status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "uptime_seconds": 3600,
    "components": {
      "database": {
        "status": "healthy",
        "message": "Database: healthy",
        "last_check": "2024-01-01T00:00:00Z",
        "response_time_ms": 5
      },
      "gpu": {
        "status": "degraded",
        "message": "GPU: disabled",
        "last_check": "2024-01-01T00:00:00Z",
        "response_time_ms": 2
      },
      "websocket": {
        "status": "healthy",
        "message": "WebSocket connections: 0",
        "last_check": "2024-01-01T00:00:00Z",
        "response_time_ms": 1
      },
      "external_apis": {
        "status": "healthy",
        "message": "External APIs: CF:OK MR:OK",
        "last_check": "2024-01-01T00:00:00Z",
        "response_time_ms": 150
      }
    },
    "version": "1.0.0"
  }
}
```

#### GET /api/system/metrics

Get system resource metrics.

**Response:**
```json
{
  "success": true,
  "data": {
    "cpu_usage": 45.2,
    "memory_usage": 60.8,
    "disk_usage": 25.3,
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### Server Management

#### GET /api/servers

List all servers.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "server-123",
      "name": "My Server",
      "minecraft_version": "1.20.1",
      "loader": "vanilla",
      "status": "running",
      "port": 25565,
      "memory_mb": 2048,
      "created_at": "2024-01-01T00:00:00Z",
      "last_start": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### POST /api/servers

Create a new server.

**Request Body:**
```json
{
  "name": "My Server",
  "minecraft_version": "1.20.1",
  "loader": "vanilla",
  "memory_mb": 2048,
  "port": 25565,
  "jvm_args": "-Xmx2G -Xms1G",
  "auto_start": false,
  "auto_restart": false,
  "modpack": {
    "id": "modpack-123",
    "provider": "modrinth",
    "version": "1.0.0"
  },
  "individual_mods": [
    {
      "id": "jei",
      "provider": "modrinth",
      "version": "1.0.0"
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "name": "My Server",
    "status": "creating",
    "creation_progress": 0.0
  }
}
```

#### GET /api/servers/{id}

Get server details.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "name": "My Server",
    "minecraft_version": "1.20.1",
    "loader": "vanilla",
    "status": "running",
    "port": 25565,
    "memory_mb": 2048,
    "created_at": "2024-01-01T00:00:00Z",
    "last_start": "2024-01-01T00:00:00Z",
    "players_online": 5,
    "tps": 20.0,
    "heap_mb": 1024
  }
}
```

#### POST /api/servers/{id}/start

Start a server.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "status": "starting"
  }
}
```

#### POST /api/servers/{id}/stop

Stop a server.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "status": "stopping"
  }
}
```

#### POST /api/servers/{id}/restart

Restart a server.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "status": "restarting"
  }
}
```

#### DELETE /api/servers/{id}

Delete a server.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "server-123",
    "status": "deleted"
  }
}
```

#### POST /api/servers/validate

Validate server creation parameters.

**Request Body:**
```json
{
  "name": "My Server",
  "minecraft_version": "1.20.1",
  "loader": "vanilla",
  "memory_mb": 2048,
  "port": 25565
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "valid": true,
    "errors": [],
    "warnings": [],
    "java_detected": {
      "version": "17.0.2",
      "path": "C:\\Program Files\\Java\\jdk-17.0.2\\bin\\java.exe",
      "compatible": true
    }
  }
}
```

### Mod Management

#### GET /api/mods/search

Search for mods.

**Query Parameters:**
- `query` (string): Search query
- `provider` (string): Provider filter (all, modrinth, curseforge)
- `page` (number): Page number (default: 0)
- `limit` (number): Results per page (default: 20)

**Response:**
```json
{
  "success": true,
  "data": {
    "mods": [
      {
        "id": "jei",
        "name": "Just Enough Items",
        "description": "JEI is an item and recipe viewing mod",
        "provider": "modrinth",
        "downloads": 1000000,
        "followers": 50000,
        "categories": ["utility"],
        "authors": ["mezz"],
        "latest_version": "1.0.0"
      }
    ],
    "total": 1,
    "page": 0,
    "limit": 20
  }
}
```

#### GET /api/mods/{id}

Get mod details.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "jei",
    "name": "Just Enough Items",
    "description": "JEI is an item and recipe viewing mod",
    "provider": "modrinth",
    "downloads": 1000000,
    "followers": 50000,
    "categories": ["utility"],
    "authors": ["mezz"],
    "latest_version": "1.0.0",
    "versions": [
      {
        "version": "1.0.0",
        "minecraft_version": "1.20.1",
        "loader": "fabric",
        "release_type": "release",
        "published": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

#### GET /api/mods/{id}/versions

Get mod versions.

**Query Parameters:**
- `minecraft_version` (string): Minecraft version filter
- `loader` (string): Loader filter

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "version": "1.0.0",
      "minecraft_version": "1.20.1",
      "loader": "fabric",
      "release_type": "release",
      "published": "2024-01-01T00:00:00Z",
      "downloads": 10000,
      "file_size": 1024000
    }
  ]
}
```

#### POST /api/mods/install

Install a mod to a server.

**Request Body:**
```json
{
  "mod_id": "jei",
  "provider": "modrinth",
  "version": "1.0.0",
  "server_id": "server-123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mod_id": "jei",
    "server_id": "server-123",
    "status": "installing"
  }
}
```

#### POST /api/mods/uninstall

Uninstall a mod from a server.

**Request Body:**
```json
{
  "mod_id": "jei",
  "server_id": "server-123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mod_id": "jei",
    "server_id": "server-123",
    "status": "uninstalled"
  }
}
```

### Modpack Management

#### GET /api/modpacks/search

Search for modpacks.

**Query Parameters:**
- `query` (string): Search query
- `provider` (string): Provider filter (all, modrinth, curseforge)
- `minecraft_version` (string): Minecraft version filter
- `loader` (string): Loader filter
- `page` (number): Page number (default: 0)
- `limit` (number): Results per page (default: 20)

**Response:**
```json
{
  "success": true,
  "data": {
    "modpacks": [
      {
        "id": "modpack-123",
        "name": "All the Mods 8",
        "description": "A large modpack with many mods",
        "provider": "modrinth",
        "downloads": 500000,
        "followers": 25000,
        "minecraft_version": "1.20.1",
        "loader": "forge",
        "latest_version": "1.0.0"
      }
    ],
    "total": 1,
    "page": 0,
    "limit": 20
  }
}
```

#### GET /api/modpacks/{id}

Get modpack details.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "modpack-123",
    "name": "All the Mods 8",
    "description": "A large modpack with many mods",
    "provider": "modrinth",
    "downloads": 500000,
    "followers": 25000,
    "minecraft_version": "1.20.1",
    "loader": "forge",
    "latest_version": "1.0.0",
    "versions": [
      {
        "version": "1.0.0",
        "minecraft_version": "1.20.1",
        "loader": "forge",
        "published": "2024-01-01T00:00:00Z",
        "downloads": 10000,
        "file_size": 1048576000
      }
    ]
  }
}
```

#### POST /api/modpacks/{id}/apply

Apply a modpack to a server.

**Request Body:**
```json
{
  "server_id": "server-123",
  "version": "1.0.0"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "modpack_id": "modpack-123",
    "server_id": "server-123",
    "status": "applying",
    "progress": 0.0
  }
}
```

### GPU Management

#### GET /api/gpu/status

Get GPU status.

**Response:**
```json
{
  "success": true,
  "data": {
    "enabled": false,
    "healthy": true,
    "worker_available": false
  }
}
```

#### GET /api/gpu/metrics

Get GPU metrics.

**Response:**
```json
{
  "success": true,
  "data": {
    "utilization": 0.75,
    "memory_used": 2048,
    "memory_total": 8192,
    "temperature": 65.5,
    "power_usage": 150.0,
    "last_update": "2024-01-01T00:00:00Z"
  }
}
```

#### POST /api/gpu/enable

Enable GPU acceleration.

**Response:**
```json
{
  "success": true,
  "data": "GPU enabled successfully"
}
```

#### POST /api/gpu/disable

Disable GPU acceleration.

**Response:**
```json
{
  "success": true,
  "data": "GPU disabled successfully"
}
```

### WebSocket Events

The API supports WebSocket connections for real-time updates.

#### Connection

Connect to: `ws://127.0.0.1:52100/ws`

#### Event Types

**Progress Events:**
```json
{
  "id": "event-123",
  "server_id": "server-123",
  "event_type": "progress",
  "data": {
    "job_id": "job-123",
    "job_type": "modpack_install",
    "status": "in_progress",
    "progress": 0.5,
    "current_step": "Installing mods",
    "total_steps": 4,
    "current_step_progress": 0.25,
    "message": "Installing mod 5 of 20",
    "error": null,
    "estimated_remaining_ms": 30000
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

**Server Status Events:**
```json
{
  "id": "event-124",
  "server_id": "server-123",
  "event_type": "server_status",
  "data": {
    "status": "running",
    "players_online": 5,
    "tps": 20.0,
    "heap_mb": 1024
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Error Handling

All API endpoints return structured error responses:

```json
{
  "success": false,
  "data": null,
  "error": "Validation failed: Server name is required",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

Common error scenarios:

- **Validation errors**: Invalid input parameters
- **Not found**: Resource doesn't exist
- **Conflict**: Resource already exists or is in use
- **Rate limited**: Too many requests
- **Internal error**: Server-side error

## Rate Limiting

Rate limits are applied per IP address:

- **Search endpoints**: 60 requests/minute
- **Download endpoints**: 30 requests/minute
- **Server management**: 10 requests/minute
- **Authentication**: 5 requests/minute

When rate limited, the API returns:

```json
{
  "success": false,
  "data": null,
  "error": "Rate limit exceeded. Try again in 60 seconds.",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Security

- All endpoints bind to localhost only by default
- Input validation prevents injection attacks
- Path sanitization prevents directory traversal
- Rate limiting prevents abuse
- No sensitive data in error responses