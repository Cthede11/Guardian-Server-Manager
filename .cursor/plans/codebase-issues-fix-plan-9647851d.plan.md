<!-- 9647851d-aa49-41d6-8ee3-9a78552db373 2b2dc48e-24f0-49f1-bbea-af8f893fa821 -->
# Fix Build Errors and Warnings

## Critical Errors to Fix

### 1. Missing websocket module reference

**File:** `hostd/src/lib.rs`

- Remove `pub mod websocket;` line (file was deleted earlier)

### 2. Platform-specific signal handling (Windows compatibility)

**File:** `hostd/src/core/shutdown.rs`

- The code uses `tokio::signal::unix` which doesn't exist on Windows
- Wrap Unix signal handlers with `#[cfg(unix)]` attribute
- Remove duplicate `setup_windows_signal_handlers` function (already exists)
- Fix function signature to match usage

### 3. Missing SecurityError variant

**File:** `hostd/src/core/credential_manager.rs` (line 259)

- Change `AppError::SecurityError` to `AppError::InternalError`
- Update error handling to use existing error variants

### 4. Duplicate WebSocket method definitions

**File:** `hostd/src/websocket_manager.rs`

- Remove duplicate definitions of `send_job_started`, `send_job_progress`, `send_job_completed`, `send_job_failed`
- Keep only the new implementations we added

### 5. Validation.rs type issues

**File:** `hostd/src/core/validation.rs`

- Remove `Custom` variant from `ValidationRule` enum (causes serialization issues with function pointers)
- Fix `FieldError` vs `AppError` type mismatches in validation logic
- Fix `contains` method call with proper type conversion

### 6. Missing imports and dependencies

**File:** `hostd/src/core/shutdown.rs`

- Add missing `use futures;` for join_all
- Fix closure lifetime issues in shutdown operations

### 7. WebSocket message pattern matching

**File:** `hostd/src/websocket_manager.rs`

- Add missing `ProgressEvent` pattern in handle_message match statement

### 8. Monitoring manager method

**File:** `hostd/src/core/crash_watchdog.rs`

- Remove or fix `record_event` call on MonitoringManager (method doesn't exist)

### 9. Serialization issues

**Files:** `hostd/src/core/crash_watchdog.rs`, `hostd/src/core/monitoring.rs`

- Remove `#[derive(Serialize)]` from structs containing `std::time::Instant`
- Use `#[serde(skip)]` or convert to serializable types

### 10. Type mismatches in retry_backoff.rs

**File:** `hostd/src/core/retry_backoff.rs`

- Fix closure return type issues in circuit breaker
- Add proper error conversion for anyhow errors

## Implementation Steps

1. Fix module declarations and imports
2. Fix platform-specific code with proper cfg attributes
3. Remove duplicate method definitions
4. Fix validation.rs type issues
5. Fix serialization issues with Instant
6. Fix missing method calls
7. Fix type mismatches and conversions
8. Address all warnings by adding underscores or removing unused code

## Files to Modify

- `hostd/src/lib.rs`
- `hostd/src/core/shutdown.rs`
- `hostd/src/core/credential_manager.rs`
- `hostd/src/websocket_manager.rs`
- `hostd/src/core/validation.rs`
- `hostd/src/core/crash_watchdog.rs`
- `hostd/src/core/retry_backoff.rs`
- `hostd/src/core/monitoring.rs`
- Various files with unused variable warnings

### To-dos

- [ ] Implement database transactions with rollback for multi-step operations (server creation, mod installation, etc.)
- [ ] Fix race conditions in ProcessManager with atomic check-and-set operations for server state
- [ ] Add port conflict detection with registry to prevent servers from using same ports
- [ ] Fix server deletion to be atomic and validate server is stopped before deletion
- [ ] Replace hardcoded credentials with secure generation and storage
- [ ] Implement websocket reconnection logic and heartbeat mechanism
- [ ] Improve crash detection to report immediately and handle errors gracefully
- [ ] Add retry logic and circuit breaker pattern for frontend API calls
- [ ] Implement console streaming to forward stdout/stderr to websocket and database
- [ ] Fix monitoring task cleanup to prevent memory leaks
- [ ] Consolidate duplicate ProcessManager and WebSocket implementations
- [ ] Add comprehensive input validation for all API endpoints
- [ ] Improve error messages to be user-friendly and actionable
- [ ] Implement proper shutdown signal handling and resource cleanup
- [ ] Add exponential backoff for crash recovery and failed operations