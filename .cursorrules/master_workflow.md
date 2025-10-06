# Guardian Server Manager – Full Production Workflow (No-Commit Edition)

---

## Mission
Bring Guardian-Server-Manager to **full production quality**:
a stable, GPU-accelerated Minecraft server manager with
Crash-proof backend, Modrinth/CurseForge integration,
complete UI, and consistent user experience.

Cursor should iteratively apply and test each phase **without committing**.
All progress and notes go into `docs/ai-logs/DEVLOG.md`.

---

## PHASE 0 – FOUNDATION
### Goals
- Verify backend (Rust), frontend (Tauri/React), Java agent, and GPU worker all build locally.
- Set up `.env` keys (CurseForge, Modrinth) and config loader.
- Implement centralized logging (`tracing`) and error handling.

### Tasks
- `cargo build` + `npm run build` baseline
- Create `guardian_config.rs` to parse `.env`
- Ensure clean `cargo clippy --deny warnings`

### Output
Stable local build; documented setup steps in `docs/DEVLOG.md`.

---

## PHASE 1 – CORE BACKEND
### Implement
- **Server lifecycle:** create/start/stop/restart/delete.
- **Crash watchdog:** detect hangs > 5 s → pause entity or restart gracefully.
- **Backups & restore:** zip world folder on stop; restore UI option.
- **Scheduler:** run backups/restarts via cron syntax.
- **Resource monitor:** CPU, RAM, GPU endpoints.

### Directives
- Use `tokio` async tasks; no blocking.
- Expose REST + WebSocket endpoints under `/api/server/*`.
- Add internal test harness (no commits).

---

## PHASE 2 – EXTERNAL API INTEGRATION
### Implement
1. **CurseForge & Modrinth** real HTTP clients:
   - `/mods/search`, `/mods/{id}`, `/files`, `.mrpack` parsing.
2. **Unified ModProvider trait** for both sources.
3. **Modpack Installer & Updater:**
   - Parse manifest.json or `.mrpack`.
   - Download server-side mods concurrently.
   - Skip client-only files.
   - Track versions for update check.

### Output
Functional API integration verified by sample test scripts.

---

## PHASE 3 – GPU ACCELERATION
### Implement
- WebGPU (wgpu) base + optional CUDA backend.
- Parallel chunk pre-generation and lighting.
- Adaptive offload based on CPU usage.
- Safe fallback to CPU.
- Record metrics to `guardian-gpu.log`.

### UI Hooks
- GPU Utilization chart in Dashboard.
- Toggle “Enable GPU tasks” in Settings.

---

## PHASE 4 – COMPATIBILITY & ANALYTICS
### Implement
- Parse `mods.toml` / `fabric.mod.json` for deps & conflicts.
- JSON ruleset of known incompatibilities.
- Recommend fixes (remove/update/install).
- Performance telemetry (TPS, tick ms, mem, IO).
- Simple heuristic “risk score” predictor.

### UI Hooks
- “Compatibility” page listing issues + auto-fix buttons.
- “Analytics” tab showing graphs.

---

## PHASE 5 – UI / UX COMPLETE REBUILD
### Required Pages
| Page | Features |
|------|-----------|
| **Dashboard** | Server list, start/stop, live metrics |
| **Server Detail** | Console stream, mod list, players, actions |
| **Mod Browser** | Unified search, filters, add/remove |
| **Modpack Manager** | Install/update/remove packs |
| **Compatibility** | Conflict list + fixes |
| **Settings** | API keys, GPU toggle, defaults |
| **Backups** | View/restore archives |
| **First-Run Wizard** | API keys, Java path, default dirs |

### Design Rules
- Dark theme `#0D1117` bg, cyan `#00BFFF` accent.
- Fonts: Inter (UI), JetBrains Mono (console).
- Rounded-2xl, shadow-md, spacing 4.
- Framer Motion animations ≤ 200 ms.
- Lucide icons; responsive ≥ 1280 px.
- Toasts for all async actions.
- Non-blocking UI – show progress indicators.

### Output
All pages implemented, theme consistent, layout responsive.

---

## PHASE 6 – TESTING & POLISH
### Tasks
- Integration tests: server start/stop, mod install, GPU jobs, backups.
- End-to-end smoke test script (`tests/e2e.rs`).
- Static analysis: `clippy`, `eslint`.
- Add `docs/ARCHITECTURE_REVIEW.md`, `API_REFERENCE.md`, `USER_GUIDE.md`.

### Definition of Done
- All endpoints return 2xx.
- `cargo test` + `npm run build` pass.
- UI fully operational.
- No hard-coded paths; all settings configurable.
- Zero console or compile warnings.

---

## QUALITY TARGETS
- Server startup < 10 s  
- GPU pregen ≥ 5× faster than CPU baseline  
- No crash on malformed modpack  
- UI ≥ 60 FPS idle / ≥ 30 FPS under load  
- No blocking API calls in UI  
- Build passes with `--release` cleanly  

---

## CODING DIRECTIVES
- **No commits, pushes, or branch changes.**
- Write to local files only.
- Log progress to `docs/ai-logs/DEVLOG.md` after each sub-phase.
- Preserve working code; never delete functioning modules.
- Keep architecture separation: UI ↔ hostd ↔ GPU ↔ agent.
- Use descriptive comments (`// AI-EXPLAIN:`) when adding new logic.
- Verify each phase with builds/tests before continuing.

---

## FINAL PRODUCT VISION
**Guardian Server Manager v1.0**
- Cross-platform Tauri app.
- GPU-accelerated world generation.
- Full mod & modpack management (CurseForge + Modrinth).
- Crash isolation, auto-restart, backups, analytics.
- Polished dark IDE-style UI.
- Responsive, stable, extensible.

---
