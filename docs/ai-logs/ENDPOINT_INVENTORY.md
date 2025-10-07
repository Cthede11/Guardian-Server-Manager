# Guardian Server Manager - API Endpoint Inventory

Generated: 2025-01-27

## Core Server Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers` | - | - | `ApiResponse<Vec<ServerInfo>>` | 500 |
| POST | `/api/servers` | `CreateServerRequest` | - | `ApiResponse<ServerInfo>` | 400, 500 |
| GET | `/api/servers/:id` | - | - | `ApiResponse<ServerInfo>` | 404, 500 |
| PATCH | `/api/servers/:id` | `UpdateServerRequest` | - | `ApiResponse<ServerInfo>` | 400, 404, 500 |
| DELETE | `/api/servers/:id` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/servers/:id/health` | - | - | `ApiResponse<ServerHealth>` | 404, 500 |
| POST | `/api/servers/:id/start` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/stop` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/restart` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/command` | `ServerCommand` | - | `ApiResponse<()>` | 400, 404, 500 |

## Resource Monitoring

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/metrics` | - | - | `ApiResponse<ServerMetrics>` | 404, 500 |
| GET | `/api/servers/:id/metrics/history` | - | `?hours=24` | `ApiResponse<Vec<MetricsSnapshot>>` | 404, 500 |
| GET | `/api/servers/:id/metrics/realtime` | - | - | `ApiResponse<RealtimeMetrics>` | 404, 500 |
| GET | `/api/system/metrics` | - | - | `ApiResponse<SystemMetrics>` | 500 |
| GET | `/api/system/metrics/history` | - | `?hours=24` | `ApiResponse<Vec<SystemMetricsSnapshot>>` | 500 |
| GET | `/api/system/resource-summary` | - | - | `ApiResponse<ResourceSummary>` | 500 |

## GPU Acceleration

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/gpu/status` | - | - | `ApiResponse<GpuStatus>` | 500 |
| GET | `/api/gpu/metrics` | - | - | `ApiResponse<GpuMetrics>` | 500 |
| POST | `/api/gpu/enable` | - | - | `ApiResponse<()>` | 500 |
| POST | `/api/gpu/disable` | - | - | `ApiResponse<()>` | 500 |
| POST | `/api/gpu/job/submit` | `GpuJobRequest` | - | `ApiResponse<GpuJob>` | 400, 500 |
| GET | `/api/gpu/job/:id/status` | - | - | `ApiResponse<GpuJobStatus>` | 404, 500 |

## Performance & Compatibility

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/performance/:server_id/metrics` | - | - | `ApiResponse<PerformanceMetrics>` | 404, 500 |
| GET | `/api/performance/:server_id/summary` | - | - | `ApiResponse<PerformanceSummary>` | 404, 500 |
| GET | `/api/performance/all` | - | - | `ApiResponse<Vec<PerformanceMetrics>>` | 500 |
| GET | `/api/compatibility/:server_id/risk-analysis` | - | - | `ApiResponse<RiskAnalysis>` | 404, 500 |
| GET | `/api/compatibility/:server_id/mod/:mod_id/risk` | - | - | `ApiResponse<ModRiskAnalysis>` | 404, 500 |
| POST | `/api/servers/:id/compat/scan` | - | - | `ApiResponse<CompatibilityScan>` | 404, 500 |
| POST | `/api/servers/:id/compat/apply` | `CompatibilityFixes` | - | `ApiResponse<()>` | 400, 404, 500 |

## Crash Watchdog

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| POST | `/api/servers/:id/watchdog/register` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/watchdog/unregister` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/servers/:id/watchdog/health` | - | - | `ApiResponse<WatchdogHealth>` | 404, 500 |
| POST | `/api/servers/:id/watchdog/force-restart` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/watchdog/heartbeat` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/watchdog/health` | - | - | `ApiResponse<Vec<WatchdogHealth>>` | 500 |

## EULA Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/eula` | - | - | `ApiResponse<EulaStatus>` | 404, 500 |
| POST | `/api/servers/:id/eula/accept` | - | - | `ApiResponse<()>` | 404, 500 |

## Server Configuration

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/config` | - | - | `ApiResponse<ServerConfig>` | 404, 500 |
| GET | `/api/servers/:id/config/server.properties` | - | - | `ApiResponse<ServerProperties>` | 404, 500 |
| PUT | `/api/servers/:id/config/server.properties` | `ServerProperties` | - | `ApiResponse<()>` | 400, 404, 500 |
| GET | `/api/servers/:id/config/jvm-args` | - | - | `ApiResponse<JvmArgs>` | 404, 500 |
| PUT | `/api/servers/:id/config/jvm-args` | `JvmArgs` | - | `ApiResponse<()>` | 400, 404, 500 |
| GET | `/api/servers/:id/settings` | - | - | `ApiResponse<ServerSettings>` | 404, 500 |

## Console & Players

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/console` | - | `?lines=100` | `ApiResponse<Vec<ConsoleMessage>>` | 404, 500 |
| GET | `/api/servers/:id/players` | - | - | `ApiResponse<Vec<Player>>` | 404, 500 |
| GET | `/api/servers/:id/players/:uuid` | - | - | `ApiResponse<Player>` | 404, 500 |
| POST | `/api/servers/:id/players/:uuid/kick` | `KickPlayerRequest` | - | `ApiResponse<()>` | 400, 404, 500 |
| POST | `/api/servers/:id/players/:uuid/ban` | `BanPlayerRequest` | - | `ApiResponse<()>` | 400, 404, 500 |

## World Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/world/freezes` | - | - | `ApiResponse<Vec<WorldFreeze>>` | 404, 500 |
| GET | `/api/servers/:id/world/heatmap` | - | `?dimension=overworld` | `ApiResponse<WorldHeatmap>` | 404, 500 |

## Backup Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/backups` | - | - | `ApiResponse<Vec<Backup>>` | 404, 500 |
| POST | `/api/servers/:id/backups` | `CreateBackupRequest` | - | `ApiResponse<Backup>` | 400, 404, 500 |
| GET | `/api/servers/:id/backups/:backup_id` | - | - | `ApiResponse<Backup>` | 404, 500 |
| POST | `/api/servers/:id/backups/:backup_id/restore` | - | - | `ApiResponse<()>` | 404, 500 |
| DELETE | `/api/servers/:id/backups/:backup_id` | - | - | `ApiResponse<()>` | 404, 500 |

## Testing

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| POST | `/api/test/run` | - | - | `ApiResponse<TestRun>` | 500 |
| POST | `/api/test/run/:test_name` | - | - | `ApiResponse<TestRun>` | 404, 500 |
| GET | `/api/test/results` | - | - | `ApiResponse<Vec<TestResult>>` | 500 |

## Modpack Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/modpacks/versions` | - | - | `ApiResponse<Vec<String>>` | 500 |
| GET | `/api/modpacks/loaders` | - | - | `ApiResponse<Vec<LoaderInfo>>` | 500 |
| GET | `/api/modpacks` | - | - | `ApiResponse<Vec<Modpack>>` | 500 |
| POST | `/api/modpacks` | `CreateModpackRequest` | - | `ApiResponse<Modpack>` | 400, 500 |
| GET | `/api/modpacks/:id` | - | - | `ApiResponse<Modpack>` | 404, 500 |
| PUT | `/api/modpacks/:id` | `UpdateModpackRequest` | - | `ApiResponse<Modpack>` | 400, 404, 500 |
| DELETE | `/api/modpacks/:id` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/modpacks/:id/apply` | `ApplyModpackRequest` | - | `ApiResponse<()>` | 400, 404, 500 |
| GET | `/api/modpacks/:id/download` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/modpacks/search` | - | `?q=term` | `ApiResponse<Vec<Modpack>>` | 500 |

## Mod Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/modpacks/mods` | - | `?q=term` | `ApiResponse<Vec<Mod>>` | 500 |
| GET | `/api/modpacks/mods/:id` | - | - | `ApiResponse<Mod>` | 404, 500 |
| GET | `/api/modpacks/mods/:id/versions` | - | - | `ApiResponse<Vec<ModVersion>>` | 404, 500 |
| GET | `/api/modpacks/mods/:id/compatibility` | - | - | `ApiResponse<ModCompatibility>` | 404, 500 |
| GET | `/api/mods/search/external` | - | `?q=term` | `ApiResponse<Vec<ExternalMod>>` | 500 |
| GET | `/api/mods/:id/compatibility` | - | - | `ApiResponse<ModCompatibility>` | 404, 500 |
| GET | `/api/servers/:id/mods` | - | - | `ApiResponse<Vec<ServerMod>>` | 404, 500 |
| POST | `/api/servers/:id/mods/plan` | `CreateModPlanRequest` | - | `ApiResponse<ModPlan>` | 400, 404, 500 |
| GET | `/api/servers/:id/mods/plan/:plan_id` | - | - | `ApiResponse<ModPlan>` | 404, 500 |
| DELETE | `/api/servers/:id/mods/plan/:plan_id` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/mods/plan/:plan_id/apply` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/mods/plan/:plan_id/rollback` | - | - | `ApiResponse<()>` | 404, 500 |

## Loader Management

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/loaders/java/detect` | - | - | `ApiResponse<JavaInfo>` | 500 |
| GET | `/api/loaders/fabric/versions` | - | - | `ApiResponse<Vec<FabricVersion>>` | 500 |
| GET | `/api/loaders/quilt/versions` | - | - | `ApiResponse<Vec<QuiltVersion>>` | 500 |
| GET | `/api/loaders/forge/versions` | - | - | `ApiResponse<Vec<ForgeVersion>>` | 500 |

## Settings & Configuration

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/settings` | - | - | `ApiResponse<Settings>` | 500 |
| PUT | `/api/settings` | `Settings` | - | `ApiResponse<()>` | 400, 500 |
| POST | `/api/settings/validate/java` | `JavaValidationRequest` | - | `ApiResponse<ValidationResult>` | 400, 500 |
| POST | `/api/settings/validate/api-keys` | `ApiKeysValidationRequest` | - | `ApiResponse<ValidationResult>` | 400, 500 |

## Server Creation Wizard

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/server/versions` | - | - | `ApiResponse<Vec<String>>` | 500 |
| POST | `/api/server/validate` | `ServerConfigValidation` | - | `ApiResponse<ValidationResult>` | 400, 500 |
| GET | `/api/server/detect-java` | - | - | `ApiResponse<JavaInfo>` | 500 |
| GET | `/api/mods/search` | - | `?q=term` | `ApiResponse<Vec<Mod>>` | 500 |
| POST | `/api/modpacks/apply` | `ApplyModpackRequest` | - | `ApiResponse<()>` | 400, 500 |
| POST | `/api/mods/install` | `InstallModsRequest` | - | `ApiResponse<()>` | 400, 500 |

## Hot Import

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/import` | - | - | `ApiResponse<Vec<HotImportJob>>` | 404, 500 |
| POST | `/api/servers/:id/import` | `CreateHotImportJobRequest` | - | `ApiResponse<HotImportJob>` | 400, 404, 500 |
| GET | `/api/servers/:id/import/:job_id` | - | - | `ApiResponse<HotImportJob>` | 404, 500 |
| DELETE | `/api/servers/:id/import/:job_id` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/import/:job_id/start` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/import/:job_id/cancel` | - | - | `ApiResponse<()>` | 404, 500 |

## Lighting Optimization

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/servers/:id/lighting` | - | - | `ApiResponse<Vec<LightingJob>>` | 404, 500 |
| POST | `/api/servers/:id/lighting` | `CreateLightingJobRequest` | - | `ApiResponse<LightingJob>` | 400, 404, 500 |
| GET | `/api/servers/:id/lighting/:job_id` | - | - | `ApiResponse<LightingJob>` | 404, 500 |
| DELETE | `/api/servers/:id/lighting/:job_id` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/lighting/:job_id/start` | - | - | `ApiResponse<()>` | 404, 500 |
| POST | `/api/servers/:id/lighting/:job_id/cancel` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/servers/:id/lighting/settings` | - | - | `ApiResponse<LightingSettings>` | 404, 500 |
| PUT | `/api/servers/:id/lighting/settings` | `LightingSettings` | - | `ApiResponse<()>` | 400, 404, 500 |

## Authentication

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| POST | `/api/auth/login` | `LoginRequest` | - | `ApiResponse<LoginResponse>` | 400, 500 |
| POST | `/api/auth/register` | `RegisterRequest` | - | `ApiResponse<LoginResponse>` | 400, 500 |
| POST | `/api/auth/logout` | - | - | `ApiResponse<()>` | 500 |
| GET | `/api/auth/me` | - | - | `ApiResponse<User>` | 401, 500 |
| GET | `/api/auth/users` | - | - | `ApiResponse<Vec<User>>` | 500 |
| GET | `/api/auth/users/:id` | - | - | `ApiResponse<User>` | 404, 500 |
| PUT | `/api/auth/users/:id` | `UserUpdate` | - | `ApiResponse<User>` | 400, 404, 500 |
| DELETE | `/api/auth/users/:id` | - | - | `ApiResponse<()>` | 404, 500 |
| GET | `/api/auth/roles` | - | - | `ApiResponse<Vec<Role>>` | 500 |
| GET | `/api/auth/permissions` | - | - | `ApiResponse<Vec<Permission>>` | 500 |

## Health & Status

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/api/health` | - | - | `ApiResponse<HealthStatus>` | 500 |
| GET | `/api/healthz` | - | - | `ApiResponse<HealthStatus>` | 500 |
| GET | `/healthz` | - | - | `ApiResponse<HealthStatus>` | 500 |
| GET | `/api/status` | - | - | `ApiResponse<SystemStatus>` | 500 |

## WebSocket

| METHOD | PATH | ReqBody | Query | Resp<T> | Errors |
|--------|------|---------|-------|---------|--------|
| GET | `/ws` | - | - | WebSocket | - |

## Response Types

All endpoints return `ApiResponse<T>` with the following structure:
```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}
```

## Error Codes

- **400**: Bad Request - Invalid input data
- **401**: Unauthorized - Authentication required
- **404**: Not Found - Resource not found
- **500**: Internal Server Error - Server-side error

## Notes

- All timestamps are in RFC3339 format
- UUIDs are in standard format
- Query parameters are URL-encoded
- Request/Response bodies are JSON
- WebSocket messages are JSON-encoded