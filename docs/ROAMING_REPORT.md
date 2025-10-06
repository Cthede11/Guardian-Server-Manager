# Roaming Report - Class B/C Proposals

## Proposals

### Proposal 1: Frontend Build System Issues (Class B)
**Status:** OPEN  
**Context:** The frontend build is failing due to missing modules and dependencies. Many TypeScript imports cannot be resolved.  
**Files:** guardian-ui/src/**/*.tsx, guardian-ui/src/**/*.ts  
**Risk Level:** Medium  
**Suggested Approach:** 
1. Generate missing type definitions from backend API
2. Create missing lib modules (client, websocket, types.gen, etc.)
3. Fix import paths and dependencies
4. Ensure proper TypeScript configuration

**Test Impact:** Frontend build will need to pass before deployment  
**UI Impact:** High - affects entire frontend functionality

### Proposal 2: Rust Edition Compatibility (Class B)
**Status:** OPEN  
**Context:** Backend build fails due to dependency requiring Rust edition 2024, which is not stable in current Cargo version.  
**Files:** hostd/Cargo.toml, hostd/src/**/*.rs  
**Risk Level:** Medium  
**Suggested Approach:**
1. Update Cargo to nightly version or wait for stable edition 2024
2. Pin problematic dependencies to older versions
3. Consider alternative dependencies that don't require edition 2024

**Test Impact:** Backend build and tests need to pass  
**UI Impact:** None - backend only

### Proposal 3: Monitoring Integration (Class B)
**Status:** OPEN  
**Context:** API endpoints use hardcoded values instead of real monitoring data. MonitoringManager exists but not integrated with API AppState.  
**Files:** hostd/src/api.rs, hostd/src/core/app_state.rs  
**Risk Level:** Low  
**Suggested Approach:**
1. Add monitoring_manager to API AppState
2. Update API endpoints to use real monitoring data
3. Implement proper error handling for monitoring failures

**Test Impact:** API responses will change from hardcoded to real data  
**UI Impact:** Medium - affects metrics display accuracy

### Proposal 4: RCON Integration (Class B)
**Status:** OPEN  
**Context:** Console message sending and player count retrieval are not implemented. RCON client exists but not integrated.  
**Files:** hostd/src/api.rs, hostd/src/core/process_manager.rs  
**Risk Level:** Medium  
**Suggested Approach:**
1. Integrate RCON client with API endpoints
2. Implement proper error handling for RCON failures
3. Add connection pooling and retry logic

**Test Impact:** Console and player endpoints need integration tests  
**UI Impact:** High - affects server management functionality

### Proposal 5: GPU Manager Integration (Class B)
**Status:** OPEN  
**Context:** GPU metrics and pregen job management are not properly integrated with the GPU manager.  
**Files:** hostd/src/api.rs, hostd/src/gpu_manager.rs  
**Risk Level:** Medium  
**Suggested Approach:**
1. Integrate GPU manager with API endpoints
2. Implement proper job queuing and status tracking
3. Add GPU capability detection and fallback

**Test Impact:** GPU-related endpoints need testing  
**UI Impact:** Medium - affects pregen and GPU features

<!-- Roamer will add Class B/C proposals here with status: OPEN, APPROVED (by human), BLOCKED, DONE -->
