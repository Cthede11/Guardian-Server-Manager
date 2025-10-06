# Cursor Background Agent – ROAMER MODE (No-Commit)

## Mission
Continuously improve Guardian-Server-Manager while I work with a main Cursor agent.
Operate as a safe “roamer” that:
- Finds and fixes real issues (backend/frontend/gpu-worker).
- Replaces placeholder/temporary code with production implementations.
- Wires missing endpoints and UI, improves error handling and UX polish.
- Surfaces larger refactors as proposals instead of executing them.
- **Never commits or pushes.**

Follow the product vision and coding directives from `.cursorrules/master_workflow.md`.

---

## Coordination & Safety

- **Sources of truth**
  - Vision & rules: `.cursorrules/master_workflow.md`
  - Human task queue: `docs/TODO_NEXT.md`
  - Locks: “LOCK:” lines in `docs/TODO_NEXT.md` and inline file banner `FOREGROUND_CLAIM` comments.

- **No-Commit**
  - Make local edits only. Do not create or switch branches. Do not run git operations.

- **Do not touch** files or tasks marked as:
  - `LOCK:` lines in the **Locks** section of `docs/TODO_NEXT.md` (with optional expiry).
  - A file containing the comment banner: `// FOREGROUND_CLAIM` or `/* FOREGROUND_CLAIM */`.

- **Change classes**
  - **Class A (auto-fix)**: safe, localized changes (compile errors, lints, types, replacing obvious `todo!()`, wiring API response typing, null/undefined guards, loading/empty/error states, form validation, progress streaming, retry/backoff, clear error messages, missing input validation).
  - **Class B (proposal-first)**: public API shape changes, DB schema, cross-module refactors, routing reorgs, redesign of core flows, CUDA interface changes. → Create a proposal entry in `docs/ROAMING_REPORT.md` and add a task candidate to `docs/TODO_NEXT.md` under “Auto-added candidates”. Do **not** execute until a human moves it into the top queue.
  - **Class C (forbidden)**: security-sensitive changes, license changes, telemetry/analytics opt-out defaults, destructive migrations, deleting working feature flags → Report only.

---

## Search & Discover (what to hunt)

Scan repo continuously for:
- **Placeholders & stubs**
  - Rust: `todo!()`, `unimplemented!()`, `panic!("TODO")`, `// TODO`, `// FIXME`, `#[allow(dead_code)]`
  - TS/JS: `// TODO`, `// FIXME`, `throw new Error("NotImplemented")`, empty handlers, missing returns
  - “PLACEHOLDER”, “STUB”, “MOCK”
- **Unimplemented endpoints**
  - Router registrations without real handlers, 501/`todo!()`, or returns with dummy data
- **Error handling gaps**
  - Missing `Result` mapping → HTTP status, silent catches, no user feedback in UI
- **Type/lint/build issues**
  - `cargo clippy --deny warnings`, `cargo test`, `npm run typecheck`, `npm run build`
- **UI/UX gaps**
  - Blocking calls on UI thread, no progress indicators, no empty/loading/error state
- **Docs drift**
  - Missing or stale endpoint docs, missing environment validation or setup steps

---

## Loop (roaming cycle)

1) **Respect locks**: load `docs/TODO_NEXT.md` and record all `LOCK:` entries.
2) **Discover**: run scans above; build a candidate list of Class A/B/C items.
3) **Apply Class A items immediately** (unless locked files):
   - Implement or replace small placeholders with production code.
   - Wire endpoints, add validation, error mapping, progress streaming.
   - Fix lints/types/builds.
   - Keep diffs surgical.
4) **For Class B items**:
   - Write a proposal section in `docs/ROAMING_REPORT.md` with:
     - Context, files, risk level, suggested approach, test impact, UI impact.
   - Append a new task under “Auto-added candidates” in `docs/TODO_NEXT.md` (unchecked).
5) **Tests & Quality gates** (after each batch of changes):
   - `cargo clippy --deny warnings`
   - `cargo test`
   - `npm run typecheck`
   - `npm run build`
   - If failing → fix or revert only the failing portion; if blocked, add `BLOCKED:` note under the relevant proposal in `docs/ROAMING_REPORT.md`.
6) **Log**: append a concise entry to `docs/ai-logs/ROAMING_LOG.md`:
   - “ACTIONS:” list, “FILES:” list, “NEXT:” hints.
7) **Repeat** indefinitely.

---

## Scope

**Allowed paths**
- `backend/**`        # hostd, APIs, schedulers, backups, compat, providers
- `frontend/**`       # Tauri/React, pages, components, hooks, styles
- `gpu-worker/**`     # GPU worker, metrics, adapters
- `docs/**`
- `scripts/**`
- `.env*`, config files

**Disallowed (unless a task says CLAIM)**
- `java-agent/**`
- packaging/installer scripts
- repository VCS configuration

---

## Quality bars & UX

- All touched endpoints return appropriate 2xx/4xx/5xx with structured error bodies.
- UI never blocks on long operations; show progress (WS if available), toasts, retry.
- Forms: validate before enabling primary action; surface field-level errors.
- Logs: use structured `tracing` in Rust; meaningful messages in UI console.
- No new warnings in Rust/TS; no unused exports/imports.

---

## Reporting

- **docs/ai-logs/ROAMING_LOG.md**: chronological actions + summaries
- **docs/ROAMING_REPORT.md**: proposals (Class B/C), with status:
  - `OPEN`, `APPROVED (by human)`, `BLOCKED`, `DONE`

