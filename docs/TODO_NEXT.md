# TODO Next - Human Task Queue

## Locks
<!-- Add LOCK: entries here with optional expiry timestamps -->

## Auto-added candidates
<!-- Roamer will add Class B/C proposals here as unchecked items -->

- [ ] Fix frontend build system issues (missing modules, dependencies, TypeScript errors)
- [ ] Resolve Rust edition compatibility issues (dependency requires edition 2024)
- [ ] Integrate monitoring manager with API endpoints for real metrics data
- [ ] Implement RCON integration for console commands and player count
- [ ] Integrate GPU manager with API endpoints for pregen job management
- [ ] Fix remaining TypeScript type mismatches (property naming: blue_green vs blueGreen, max_players vs maxPlayers)
- [ ] Enhance ServerSummary interface with missing properties (tps, players_online, heap_mb, memory, tick_p95_ms)
- [ ] Restructure ServerSettings interface to include nested objects (general, jvm, gpu, composer, ha, paths, tokens)
- [ ] Standardize property naming conventions between frontend and backend
- [ ] Complete API contract audit between frontend and backend
- [ ] Implement error handling improvements and validation
- [ ] Add loading/empty/error states to UI components
- [ ] Modpack System API Restructuring - align frontend types with backend expectations
- [ ] Settings System Enhancement - add missing general configuration structure
- [ ] API Client Method Signature Alignment - fix method signatures and error handling
