# Guardian QA Testing Plan

## Overview
This document outlines the comprehensive testing plan for Guardian, focusing on the zero-downtime scenario and end-to-end functionality.

## Test Scenarios

### 1. Zero-Downtime Scenario Test
**Objective**: Verify that all core features work together without requiring server restarts.

**Steps**:
1. **Setup**
   - Install Guardian desktop application
   - Configure API keys (CurseForge, Modrinth)
   - Set up Java path and default settings

2. **Server Creation**
   - Create a new Minecraft server (Vanilla/Fabric/Forge/NeoForge)
   - Verify server starts successfully
   - Check that server appears in the UI with correct status

3. **Mod Integration Tool**
   - Add some mods to the server
   - Run compatibility scan
   - Verify conflicts are detected
   - Apply auto-fixes
   - Verify server still starts without errors

4. **GPU Pre-generation**
   - Create a pre-generation job
   - Configure radius based on max players
   - Start GPU-accelerated pre-generation
   - Monitor progress via WebSocket
   - Verify chunks are generated correctly

5. **Hot Import**
   - Create a hot import job
   - Import pre-generated chunks while server is running
   - Verify no server restart is required
   - Check that players can explore new areas smoothly

6. **Lighting Optimization**
   - Run lighting optimization on pre-generated chunks
   - Verify performance improvement
   - Check that lighting is correct

7. **Mod Management**
   - Search for new mods
   - Create installation plan
   - Apply mod changes transactionally
   - Verify rollback works if needed

8. **Backup and Restore**
   - Create a backup of the server
   - Verify backup is created successfully
   - Test restore functionality
   - Verify server works after restore

### 2. Performance Testing

**GPU Worker Performance**:
- Test chunk generation rate with GPU acceleration
- Compare with CPU fallback performance
- Verify memory usage is reasonable
- Check for memory leaks during long operations

**WebSocket Streaming**:
- Test real-time updates during operations
- Verify no UI freezing during large data transfers
- Check connection stability during long operations

**Database Performance**:
- Test with large numbers of servers
- Verify query performance with many mods
- Check migration performance

### 3. Error Handling Testing

**Network Failures**:
- Test behavior when API keys are invalid
- Verify graceful handling of network timeouts
- Check recovery from connection drops

**File System Errors**:
- Test behavior when disk space is low
- Verify handling of permission errors
- Check recovery from file corruption

**GPU Failures**:
- Test fallback to CPU when GPU is unavailable
- Verify graceful degradation
- Check error reporting

### 4. Cross-Platform Testing

**Windows**:
- Test on Windows 10/11
- Verify all features work
- Check installer functionality

**Linux**:
- Test on Ubuntu 20.04+
- Verify GPU acceleration works
- Check file permissions

### 5. Integration Testing

**External APIs**:
- Test CurseForge API integration
- Test Modrinth API integration
- Verify rate limiting handling

**Minecraft Server Integration**:
- Test with different Minecraft versions
- Verify RCON functionality
- Check server monitoring

## Test Data

### Sample Mod Packs
- **Create + Valkyrien Skies**: Known conflict pack for testing compatibility tool
- **OptiFine + Sodium**: Test for lighting optimization conflicts
- **Large Mod Pack**: Test performance with 200+ mods

### Test Servers
- **Vanilla Server**: Basic functionality testing
- **Fabric Server**: Mod loader testing
- **Forge Server**: Legacy mod support
- **NeoForge Server**: Modern Forge alternative

## Automated Testing

### Unit Tests
- Database operations
- API endpoints
- GPU worker functions
- Mod compatibility scanning

### Integration Tests
- End-to-end server creation
- Mod installation workflow
- Backup/restore cycle
- Hot import process

### Performance Tests
- Chunk generation benchmarks
- Memory usage monitoring
- Database query performance
- WebSocket throughput

## Manual Testing Checklist

### First Run Experience
- [ ] Application starts without errors
- [ ] Settings wizard appears
- [ ] API key validation works
- [ ] Java path detection works
- [ ] Database initializes correctly

### Server Management
- [ ] Create server wizard works
- [ ] Server starts/stops/restarts correctly
- [ ] Console output displays properly
- [ ] Player management works
- [ ] Server settings can be modified

### Mod Management
- [ ] Mod search works
- [ ] Compatibility scanning works
- [ ] Auto-fix suggestions are accurate
- [ ] Mod installation is transactional
- [ ] Rollback works correctly

### Pre-generation
- [ ] GPU worker initializes
- [ ] Chunk generation works
- [ ] Progress updates in real-time
- [ ] Fallback to CPU works
- [ ] Jobs can be paused/resumed

### Hot Import
- [ ] Chunks can be imported while server is running
- [ ] No server restart required
- [ ] TPS monitoring works
- [ ] Import can be paused/resumed
- [ ] Safety checks work

### Lighting Optimization
- [ ] Optimization levels work correctly
- [ ] GPU acceleration works
- [ ] Performance improvement is measurable
- [ ] Lighting remains correct

### Backup/Restore
- [ ] Backups are created successfully
- [ ] Restore works correctly
- [ ] Retention policies work
- [ ] Compression works

### Observability
- [ ] Logs are comprehensive
- [ ] Metrics are accurate
- [ ] WebSocket updates work
- [ ] Health checks work

## Success Criteria

### Zero-Downtime Scenario
- [ ] Complete workflow can be executed without server restarts
- [ ] All features work together seamlessly
- [ ] Performance is acceptable throughout
- [ ] No data corruption occurs

### Performance
- [ ] GPU pre-generation is 10x faster than CPU
- [ ] Hot import doesn't cause TPS drops > 5%
- [ ] UI remains responsive during operations
- [ ] Memory usage stays within reasonable bounds

### Reliability
- [ ] No crashes during normal operation
- [ ] Graceful handling of all error conditions
- [ ] Recovery from failures works
- [ ] Data integrity is maintained

### User Experience
- [ ] Intuitive interface
- [ ] Clear error messages
- [ ] Progress indicators work
- [ ] Settings are persistent

## Test Environment Setup

### Hardware Requirements
- **GPU**: NVIDIA GTX 1060 or better
- **RAM**: 16GB minimum
- **Storage**: 100GB free space
- **CPU**: Intel i5 or AMD Ryzen 5

### Software Requirements
- **OS**: Windows 10/11 or Ubuntu 20.04+
- **Java**: OpenJDK 17+
- **Node.js**: 18+
- **Rust**: 1.70+

### Test Data
- **World Size**: 10,000 x 10,000 blocks
- **Mod Count**: 100+ mods
- **Player Count**: 10+ concurrent players
- **Operation Duration**: 24+ hours continuous

## Reporting

### Test Results
- Pass/Fail status for each test
- Performance metrics
- Error logs and stack traces
- Screenshots of issues

### Issues Found
- Severity classification
- Steps to reproduce
- Expected vs actual behavior
- Suggested fixes

### Recommendations
- Performance optimizations
- UI/UX improvements
- Additional test cases
- Documentation updates
