# Guardian Production Implementation TODO

## Current State Assessment ✅

**What's Already Built:**
- ✅ Basic Tauri desktop app with React UI
- ✅ Rust backend (hostd) with SQLite database
- ✅ Server CRUD operations and lifecycle management
- ✅ Basic GPU worker with WebGPU (wgpu) implementation
- ✅ Mod management system with CurseForge/Modrinth integration
- ✅ WebSocket streaming for real-time updates
- ✅ Backup system foundation
- ✅ Basic pregen UI components

**What's Missing for Production Goals:**
- ❌ CUDA GPU acceleration (currently WebGPU only)
- ❌ Zero-downtime hot import system
- ❌ Mod compatibility scanning and auto-fix
- ❌ Server-driven fallback pregeneration
- ❌ Dynamic pregen radius based on max players
- ❌ Lighting optimization system
- ❌ Comprehensive settings API
- ❌ Production-ready packaging

## Implementation Plan

### Phase 1: Database & Core Infrastructure (Week 1)

#### 1.1 Database Schema Updates
- [ ] Add missing tables: `settings`, `tasks`, `pregeneration_policy`
- [ ] Add `max_players` and `pregeneration_policy` to servers table
- [ ] Create migration system for schema updates
- [ ] Add indexes for performance optimization

#### 1.2 Settings API Implementation
- [ ] Build comprehensive settings API endpoints
- [ ] Add Java path validation and auto-detection
- [ ] Implement API key validation for CurseForge/Modrinth
- [ ] Add settings UI with validation and testing

#### 1.3 Enhanced Server Management
- [ ] Add max_players field to server creation
- [ ] Implement dynamic pregen radius calculation
- [ ] Add server status monitoring improvements
- [ ] Enhance server lifecycle management

### Phase 2: GPU Acceleration & World Generation (Week 2)

#### 2.1 CUDA GPU Worker
- [ ] Replace WebGPU with CUDA implementation
- [ ] Implement CUDA kernels for noise generation
- [ ] Add density and mask generation kernels
- [ ] Create GPU capability detection and fallback

#### 2.2 Server-Driven Fallback Pregen
- [ ] Implement headless server orchestration
- [ ] Add parallel region generation
- [ ] Create staging directory management
- [ ] Add resumable job system

#### 2.3 Hot Import System
- [ ] Build chunk diff scanner
- [ ] Implement loaded chunk detection
- [ ] Add atomic file operations
- [ ] Create TPS-aware throttling

### Phase 3: Mod Integration & Compatibility (Week 3)

#### 3.1 Mod Compatibility Scanner
- [ ] Parse fabric.mod.json and mods.toml files
- [ ] Detect mixin conflicts and dependencies
- [ ] Build compatibility database
- [ ] Create auto-fix suggestions

#### 3.2 Mod Management Enhancements
- [ ] Implement transactional mod operations
- [ ] Add dependency resolution
- [ ] Create mod conflict resolution
- [ ] Add rollback capabilities

### Phase 4: Advanced Features (Week 4)

#### 4.1 Lighting Optimization
- [ ] Implement GPU lighting pass
- [ ] Add CPU fallback for lighting
- [ ] Create user controls for lighting settings
- [ ] Add performance presets

#### 4.2 Dynamic Pregen Policies
- [ ] Implement max players → radius mapping
- [ ] Add configurable radius policies
- [ ] Create pregen planning wizard
- [ ] Add efficiency package options

#### 4.3 Enhanced Observability
- [ ] Add comprehensive logging
- [ ] Implement diagnostics collection
- [ ] Create performance monitoring
- [ ] Add error reporting system

### Phase 5: Production Readiness (Week 5)

#### 5.1 Packaging & Distribution
- [ ] Complete Tauri build configuration
- [ ] Add installer scripts
- [ ] Implement first-run setup
- [ ] Create update mechanism

#### 5.2 Testing & QA
- [ ] End-to-end testing suite
- [ ] Performance benchmarking
- [ ] Compatibility testing
- [ ] User acceptance testing

## Success Criteria

### Core Functionality
- [ ] Server wizard works for Vanilla/Fabric/Forge/NeoForge
- [ ] GPU pregen works on NVIDIA with CUDA
- [ ] Server-driven fallback works for modded worlds
- [ ] Zero-downtime hot import works reliably
- [ ] Mod compatibility tool reduces boot errors

### Performance Targets
- [ ] GPU pregen: 50+ chunks/second on NVIDIA
- [ ] Hot import: <1 second per region
- [ ] Mod scan: <30 seconds for 100+ mods
- [ ] Memory usage: <2GB for backend

### User Experience
- [ ] First-run setup completes in <5 minutes
- [ ] UI responds within 100ms
- [ ] Error messages are clear and actionable
- [ ] Settings persist across restarts

## Technical Debt & Improvements

### Code Quality
- [ ] Add comprehensive error handling
- [ ] Implement proper logging throughout
- [ ] Add unit tests for critical paths
- [ ] Refactor duplicate code

### Performance
- [ ] Optimize database queries
- [ ] Implement caching where appropriate
- [ ] Add connection pooling
- [ ] Optimize memory usage

### Security
- [ ] Sanitize all user inputs
- [ ] Implement proper API key storage
- [ ] Add rate limiting
- [ ] Secure file operations

## Risk Mitigation

### Technical Risks
- **CUDA compatibility**: Implement WebGPU fallback
- **Performance issues**: Add comprehensive monitoring
- **Memory leaks**: Implement proper resource cleanup
- **File corruption**: Add atomic operations and validation

### User Experience Risks
- **Complex setup**: Create guided wizards
- **Performance issues**: Add progress indicators
- **Error handling**: Implement clear error messages
- **Data loss**: Add backup and recovery

## Timeline

- **Week 1**: Database & Core Infrastructure
- **Week 2**: GPU Acceleration & World Generation  
- **Week 3**: Mod Integration & Compatibility
- **Week 4**: Advanced Features
- **Week 5**: Production Readiness & Testing

**Total Estimated Time**: 5 weeks
**Target Completion**: End of February 2025
