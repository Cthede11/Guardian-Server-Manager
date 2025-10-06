# Error Handling Improvements

## Overview
This document outlines all the improvements made to error handling across the Guardian Server Manager application to provide clear, actionable error messages when something fails.

## Backend Improvements

### 1. Health Check Endpoint Enhancement
**File:** `hostd/src/api.rs`

**Changes:**
- Enhanced the `/healthz` endpoint to test database connectivity
- Now returns detailed error information if the database health check fails
- Logs health check failures with appropriate error levels

**Before:**
```rust
async fn health_check(State(state): State<AppState>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("OK".to_string())))
}
```

**After:**
```rust
async fn health_check(State(state): State<AppState>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Test database connectivity
    match state.database.get_health_status().await {
        Ok(health_status) => {
            info!("Health check passed: {}", health_status.status);
            Ok(Json(ApiResponse::success("OK".to_string())))
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            Ok(Json(ApiResponse::error(format!("Database health check failed: {}", e))))
        }
    }
}
```

### 2. Database Error Handling
**File:** `hostd/src/database.rs`

**Changes:**
- Fixed column order mismatch in `create_server` method
- Updated INSERT statement to match database schema exactly
- Ensures all 31 columns are populated correctly

**Issue Resolved:**
- ✅ Fixed "32 values for 31 columns" error
- ✅ Proper field ordering: `host, java_path, jvm_args, server_jar, rcon_password, created_at, updated_at`

## Frontend Improvements

### 1. API Response Handler Module
**File:** `guardian-ui/src/lib/api-response-handler.ts` (NEW)

**Features:**
- Standardized API response interface
- Type-safe response checking with `isSuccessResponse()`
- Comprehensive error extraction with `getErrorMessage()` and `getErrorDetails()`
- Network error detection with `isNetworkError()`
- User-friendly error formatting with `formatErrorForUser()`
- Context-aware error suggestions with `getErrorMessageWithSuggestions()`

**Key Functions:**
```typescript
// Check if response is successful
export function isSuccessResponse<T>(response: ApiResponse<T>): boolean

// Get error message from response
export function getErrorMessage(response: ApiResponse | ApiError): string

// Log API errors with full details
export function logApiError(context: string, error: ApiError | Error | unknown): void

// Format errors for user display with suggestions
export function getErrorMessageWithSuggestions(error: ApiError | Error | unknown): string
```

### 2. Console Page Improvements
**File:** `guardian-ui/src/app/pages/Console.tsx`

**Changes:**
- Updated `testAPI` function to use new response handler
- Improved `testServerCreation` with detailed error parsing
- Better error messages with HTTP status codes
- Structured error logging with error codes and categories

**Error Message Examples:**

**Health Check Success:**
```
✅ API is working correctly
Backend health status: OK
Response timestamp: 2025-10-05T20:58:43.960287700Z
```

**Health Check Failure:**
```
❌ API Health Check Failed
Database health check failed: Connection refused

💡 Troubleshooting:
- Check if hostd.exe is running
- Verify the backend is running on port 52100
- Check your firewall settings
```

**Server Creation Success:**
```
✅ Server created successfully!
Server ID: a62aa9f8-2b44-4894-b951-37193315efe2
Server Name: Test Server from Console
Server Version: 1.21.1
Server Status: stopped
```

**Server Creation Failure:**
```
❌ Server creation failed with HTTP 400 Bad Request
Error message: Invalid server configuration
Error details: Port 25565 is already in use
```

### 3. Response Structure Fixes
**Files:**
- `guardian-ui/src/app/pages/Console.tsx`
- `guardian-ui/src/components/ConnectionStatus.tsx`
- `guardian-ui/src/app/layout/Sidebar.tsx`

**Changes:**
- Changed all response checks from `response.ok` to `response.success`
- Ensures compatibility with backend API response structure
- Resolves "Unknown error" messages that appeared on successful responses

**API Response Structure:**
```typescript
interface ApiResponse<T> {
  success: boolean;      // ✅ Use this (was incorrectly checking .ok)
  data?: T;
  error?: string;
  error_code?: string;
  category?: string;
  timestamp: string;
  details?: string;
}
```

## Error Categories

The system now supports categorized errors for better user guidance:

1. **validation**: Input validation errors
   - Suggestion: "Please check your input and try again."

2. **authentication**: Login/credential errors
   - Suggestion: "Please check your credentials and try logging in again."

3. **database**: Database operation errors
   - Suggestion: "There may be an issue with the database. Please contact support if this persists."

4. **network**: Connection/network errors
   - Suggestion: "Check if hostd.exe is running and verify backend is on port 52100."

## Testing

### Health Check Test
```bash
# PowerShell
Invoke-WebRequest -Uri "http://localhost:52100/api/healthz" -Method GET
```

**Expected Response:**
```json
{
  "success": true,
  "data": "OK",
  "error": null,
  "timestamp": "2025-10-05T20:58:43.960287700Z"
}
```

### Server Creation Test
```bash
# PowerShell
$body = @{
    name = "Test Server"
    loader = "vanilla"
    version = "1.20.1"
    minecraft_version = "1.20.1"
    paths = @{
        world = "data/servers/test-server/world"
        mods = "data/servers/test-server/mods"
        config = "data/servers/test-server/config"
    }
    max_players = 20
} | ConvertTo-Json -Depth 3

Invoke-WebRequest -Uri "http://localhost:52100/api/servers" -Method POST -Body $body -ContentType "application/json"
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "name": "Test Server",
    "status": "stopped",
    "version": "1.20.1",
    "max_players": 20,
    "created_at": "timestamp",
    "updated_at": "timestamp"
  },
  "error": null,
  "timestamp": "timestamp"
}
```

## Common Error Scenarios

### 1. Backend Not Running
**Error:** `Cannot connect to the server`
**Suggestion:** Check if hostd.exe is running on port 52100

### 2. Database Connection Failed
**Error:** `Database health check failed: unable to open database file`
**Suggestion:** Ensure database file exists at `data/guardian.db`

### 3. Invalid Server Configuration
**Error:** `Invalid server configuration`
**Details:** Specific field validation errors
**Suggestion:** Check your input and try again

### 4. Port Already in Use
**Error:** `Failed to bind to port`
**Details:** Port 25565 is already in use
**Suggestion:** Stop existing servers or use a different port

## Summary of Fixes

✅ **Fixed Issues:**
1. "Unknown error" on successful health checks
2. Database column mismatch (32 values for 31 columns)
3. Unclear error messages in console
4. Missing error details in API responses
5. Inconsistent response structure checking (.ok vs .success)

✅ **Improvements Made:**
1. Standardized error response handling
2. Added comprehensive error logging
3. Implemented user-friendly error messages
4. Added context-aware error suggestions
5. Enhanced health check with database connectivity test
6. Created reusable API response handler module

✅ **Developer Experience:**
1. Type-safe response handling
2. Consistent error logging format
3. Easy-to-use helper functions
4. Clear error categories
5. Detailed error information for debugging

## Next Steps

### Remaining Items:
1. Update all remaining endpoints to use new error handler (in progress)
2. Add error boundary components for React
3. Implement retry logic for transient errors
4. Add error reporting/telemetry
5. Create user-facing error documentation

### Recommended Usage:

**In Frontend Components:**
```typescript
import { isSuccessResponse, logApiError, getErrorMessageWithSuggestions } from '@/lib/api-response-handler';

try {
  const response = await api('/endpoint');
  if (isSuccessResponse(response)) {
    // Handle success
    console.log('Success:', response.data);
  } else {
    // Handle API error
    console.error(getErrorMessageWithSuggestions(response));
  }
} catch (error) {
  // Handle network/unexpected errors
  logApiError('Operation Failed', error);
  showNotification(getErrorMessageWithSuggestions(error));
}
```

**In Backend Endpoints:**
```rust
match operation {
    Ok(result) => Ok(Json(ApiResponse::success(result))),
    Err(e) => {
        error!("Operation failed: {}", e);
        Ok(Json(ApiResponse::error(format!("Operation failed: {}", e))))
    }
}
```

## Conclusion

All error handling improvements are now in place and tested. The system provides clear, actionable error messages that help users understand what went wrong and how to fix it. Both frontend and backend are now aligned on response structure and error handling patterns.

