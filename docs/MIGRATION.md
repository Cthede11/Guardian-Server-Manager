# Guardian UI Overhaul — MIGRATION

## Goal
Rebuild the UI to a server-centric, dark panel console (MC-Server-Soft style), with strict performance budgets and real-time streams.

## Old → New Map
| Old Route/Page                | New Route/Page                                  | Notes |
|------------------------------|--------------------------------------------------|-------|
| /dashboard                   | /servers/:id/overview                            | Per-server scope; summary cards + health |
| /logs                        | /servers/:id/console                             | Virtualized stream + command input |
| /players                     | /servers/:id/players                             | Virtualized table; row actions |
| /world                       | /servers/:id/world                               | Heatmap + Chunk/Freeze inspectors |
| /mods                        | /servers/:id/modsrules                           | Mods, Conflicts, Rules, Live Rule Lab |
| /metrics                     | /servers/:id/performance                         | Charts + budgets/policies |
| /backups                     | /servers/:id/backups                             | Timeline + restore wizard |
| /events                      | /servers/:id/events                              | Scheduler + templates |
| /pregen                      | /servers/:id/pregen                              | Region queue + GPU badge |
| /sharding                    | /servers/:id/sharding                            | Topology + assignments |
| /diagnostics                 | /servers/:id/diagnostics                         | Crashes, dumps, GC logs |
| /settings                    | /servers/:id/settings/*                          | General/JVM/GPU/HA/Paths/Composer/Tokens |
| (global settings mixed in)   | /workspace/(users|backups|tokens|theme)          | Only truly global items remain |

## New File Structure
```
src/
├── app/
│   ├── layout/
│   │   ├── AppShell.tsx          # Main layout container
│   │   ├── Sidebar.tsx           # Server list + navigation
│   │   ├── ServerHeader.tsx      # Server info + tabs
│   │   └── FooterBar.tsx         # Status bar
│   ├── pages/
│   │   ├── Servers/              # Server-specific pages
│   │   │   ├── Overview.tsx      # Dashboard
│   │   │   ├── Console.tsx       # Log stream
│   │   │   ├── Players.tsx       # Player management
│   │   │   ├── World.tsx         # World tools
│   │   │   ├── ModsRules.tsx     # Mod management
│   │   │   ├── Performance.tsx   # Metrics & charts
│   │   │   ├── Backups.tsx       # Backup management
│   │   │   ├── Events.tsx        # Event scheduler
│   │   │   ├── Pregen.tsx        # World pregeneration
│   │   │   ├── Sharding.tsx      # Sharding config
│   │   │   ├── Diagnostics.tsx   # Debug tools
│   │   │   └── Settings/         # Server settings
│   │   └── Workspace/            # Global settings
│   └── routes.tsx                # Route definitions
├── components/
│   ├── Console/
│   │   └── ConsoleStream.tsx     # Virtualized log viewer
│   ├── Tables/
│   │   ├── PlayersTable.tsx      # Virtualized player list
│   │   ├── RulesTable.tsx        # Rules management
│   │   └── SnapshotsTable.tsx    # Backup snapshots
│   ├── Charts/
│   │   ├── TpsChart.tsx          # TPS monitoring
│   │   ├── PhaseChart.tsx        # Tick phases
│   │   ├── HeapChart.tsx         # Memory usage
│   │   └── LatencyChart.tsx      # Network latency
│   └── ui/                       # Reusable UI components
├── store/
│   ├── servers.ts                # Server entities & settings
│   └── live.ts                   # Real-time data & sockets
└── lib/
    ├── socket.ts                 # WebSocket/SSE management
    ├── api.ts                    # API client
    └── types.ts                  # TypeScript definitions
```

## State & Realtime

### Zustand Stores
- **`servers.ts`**: Server entities, settings, selected server
- **`live.ts`**: Real-time data, socket connections, batched updates

### Realtime Channels
- `console:{id}` - Console log streams
- `metrics:{id}` - Performance metrics
- `players:{id}` - Player data updates
- `freezes:{id}` - World freeze events
- `pregen:{id}` - Pregeneration progress

### Update Batching
All live updates are batched via `requestAnimationFrame` to minimize React commits:
```typescript
// Socket events are queued and batched
batchUpdate(() => {
  liveStore.getState().appendConsole(serverId, lines);
  liveStore.getState().applyMetrics(serverId, data);
});
```

## Performance Budget

### Targets
- **Tab switch**: < 100ms cold, < 16ms warm
- **Console**: 5k line rolling buffer; < 1ms render per new batch
- **Players**: 2k rows virtualized; row re-renders scoped to changed rows only
- **Charts**: Max 120 points per series visible; update ≤ 1Hz

### Implementation
- Route-level code splitting with `React.lazy()`
- Virtualization with `@tanstack/react-virtual`
- Memoization with `React.memo` and `useMemo`
- Narrow Zustand selectors with `shallow` comparison

## Testing

### Console Stream
- Filter messages by level and search
- Send commands and verify responses
- Test with high-frequency log streams (1k+ lines/minute)

### Players Management
- Load 2k+ players and verify smooth scrolling
- Test player actions (kick, ban, message, teleport)
- Verify success/error notifications

### Restore Wizard
- Test backup scope validation
- Verify payload construction
- Test restore process end-to-end

## Rollout Strategy

### Phase 1: Foundation
1. Create feature branch `feat/guardian-ui-overhaul`
2. Implement new routing structure
3. Set up Zustand stores with batching
4. Create socket management system

### Phase 2: Core Features
1. Implement virtualized Console component
2. Build virtualized Players table
3. Create memoized Chart components
4. Add lazy-loaded route pages

### Phase 3: Advanced Features
1. Implement World heatmap with throttling
2. Add Settings forms with validation
3. Create error handling and notifications
4. Add performance monitoring

### Phase 4: Migration
1. Move old routes under `/legacy/*` temporarily
2. Migrate tab by tab with MSW mocks
3. QA with performance profiling
4. Remove legacy routes

## Breaking Changes

### API Changes
- Server-specific endpoints now require `:id` parameter
- Real-time data uses WebSocket/SSE instead of polling
- Response formats may differ - use typed adapters in `lib/api.ts`

### Component Changes
- All server pages are now lazy-loaded
- Large lists use virtualization
- State management moved to Zustand stores
- Real-time updates are batched

### Styling Changes
- New dark theme with CSS variables
- MC-Server-Soft inspired design
- Panel-based layout with shadows
- Status pills for quick info

## Migration Checklist

### Development
- [ ] Feature branch created
- [ ] Dependencies installed
- [ ] New routing structure implemented
- [ ] Zustand stores created
- [ ] Socket management implemented

### Components
- [ ] Console component virtualized
- [ ] Players table virtualized
- [ ] Charts memoized
- [ ] Settings forms created
- [ ] Error handling implemented

### Testing
- [ ] Console stream tested
- [ ] Players actions tested
- [ ] Restore wizard tested
- [ ] Performance budget verified
- [ ] Cross-browser compatibility checked

### Deployment
- [ ] Legacy routes preserved
- [ ] Feature flags implemented
- [ ] Monitoring added
- [ ] Rollback plan ready
- [ ] Documentation updated

## Support

### Performance Issues
- Check `PERF_NOTES.md` for optimization guide
- Use Chrome DevTools Performance tab
- Monitor memory usage and frame rates

### API Issues
- Verify endpoint compatibility in `lib/api.ts`
- Check WebSocket connection status
- Review error handling in components

### UI Issues
- Check Tailwind CSS variables
- Verify component lazy loading
- Test responsive design
