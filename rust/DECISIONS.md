# DECISIONS.md — 9router-rust whole-app port

Source of truth for the port. One ADR section per decision, numbered in order, never renumbered.
A changed decision gets a new ADR that supersedes the old one. Port to files; copy nothing here
into other files — this file is read, not duplicated.

## ADR-001: Shape — one Rust crate `rust/app`, axum front door owns the HTTP server
Status: accepted (2026-11-21)
Context: analyze --whole-app says shape is **server**, entered through 6 entry points:
`cmd/9router-go` (daemon), `cmd/9router-tray`, `cmd/changelog-merge` (CLI),
`benchmark/mock_upstream.go` (server), `benchmark/runner.go` (CLI),
`tools/sitegen/main.go` (CLI).
Decision: All 6 entry points live in `rust/app`. An axum front door owns HTTP first and proxies
every route not yet ported to the original Go server on its own port. The other entry points
become Rust binaries in `rust/app` sharing the ported units. The original Go binary stays as the
fallback until the lift.
Consequences: `rust/app` is the single front door from day one; fallback rate only goes down;
CLI args/stdin/env and HTTP routes/status codes pass through unchanged for unported parts.

## ADR-002: Scope — port every feature; Svelte UI stays outside Rust
Status: accepted (2026-11-21)
Context: 800 Go source files, 78 units, ~207k lines. The dashboard at `web/` is Svelte 5 + Vite,
embedded into the Go binary via `web/embed.go`.
Decision: Port all backend units, no feature dropped. The Svelte UI stays as-is (outside Rust),
built to `web/dist` and embedded/served by the Rust app at runtime. No UI rewrite in this port.
Consequences: `web/` is a `keep` unit per §6 of AGENTS.md; the Rust app serves `web/dist`
assets (compressed .br/.gz) from its own origin; no cross-origin assets.

## ADR-003: Equivalence bar — equal after normalizing, floats within 1e-9
Status: accepted (2026-11-21)
Context: `app record` sorts noise into values (→ `[..]`), order (→ any-order compare), lines (→ `...`).
Decision: Outputs equal after normalizing JSON key order, whitespace, float formatting. Numeric
code uses `--float-tolerance 1e-9` (floats match within that share of the larger value, absolute
below 1). Integers, strings and exit codes match exactly. HTTP status codes and response schemas
are byte-exact per the parity protocol (§3 of repo AGENTS.md).
Consequences: noise normalized in the case, never in the port. `order`/`lines` cases confirmed
with the user before treating as contract.

## ADR-004: Compatibility — keep config, env, SQLite DB, wire formats, CLI flags, HTTP routes/status codes identical
Status: accepted (2026-11-21)
Context: viper + TOML/YAML config, env vars, on-disk SQLite DB at ~/.9router/db/data.sqlite
(modernc.org/sqlite), urfave/cli v2 flags, go-chi routes.
Decision: Keep all of them identical in the Rust port. Config parsed with a TOML/YAML crate;
CLI flags with clap; HTTP routes with axum; SQLite DB file format unchanged (same file, same
schema, readable/writable by both binaries during side-by-side run).
Consequences: DB crate choice must read/write the same SQLite file; config files keep viper
shape; CLI flags keep urfave names.

## ADR-005: Dependencies & runtime — tokio + axum, hand-wired DI, no fx clone
Status: accepted (2026-11-21)
Context: go-chi/chi → axum (partial fit, routing/middleware differ), go.uber.org/fx → no direct
Rust equivalent, golang.org/x/sync/singleflight → tokio/OnceCell, github.com/samber/lo →
iterators/idiomatic Rust.
Decision: Async runtime tokio; HTTP framework axum; DI hand-wired constructors in `rust/app`
(no reflection DI crate); singleflight replaced by per-key tokio mutex/OnceCell; `lo` helpers
replaced by idiomatic Rust iterators.
Consequences: `internal/app` and `internal/service` fx wiring rebuilt by hand; chi middleware
converted to tower layers/axum middleware; `lo.Map`/`lo.Filter` become `iter.map`/`iter.filter`.

## ADR-006: Targets — Linux x86_64 + aarch64, MSRV 1.84, single static binary, no C deps
Status: accepted (2026-11-21)
Context: repo AGENTS.md §4.F requires every CROSS_TARGETS target to build with CGO_ENABLED=0;
single portable binary guarantee. rusqlite/sqlx-sqlite bundle libsqlite3 via cc → ruled out.
Decision: Linux x86_64 and aarch64 release targets (Windows/macOS maybe later), MSRV 1.84,
single static binary, no C compiler required at build. SQLite crate must build without cc.
Consequences: rules out rusqlite and sqlx-sqlite; research D4 (rust/research/modernc.org-sqlite.md)
must find a no-C SQLite crate or the DB unit stays blocked.

## ADR-007: Checkpoints — stop for go-ahead after every wave
Status: accepted (2026-11-21)
Decision: After each wave: report units done, gate results, replay results, fallback rate, what's
left; update `rust/MAP.md`; write `rust/HANDOFF.md`; wait for user go-ahead before the next wave.
Consequences: port proceeds wave by wave, lowest first, dependencies already Rust.

## ADR-008: Cutover — beside-original, then lift, then switch traffic
Status: accepted (2026-11-21)
Decision: Front door proxies unported routes to the Go original; kill switch
`RUST_ANYTHING_UNIT_<NAME>=original` per unit; when fallback rate is zero and every unit passes
`gate`, run `app lift` to delete fallback and shadows; then cut production traffic via
`rust/cutover.sh`; original kept as reference until the user says otherwise.
Consequences: original Go binary remains in the repo as reference for the lifetime of the port.
