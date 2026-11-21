# MAP.md — 9router-rust whole-app port plan

Goal: a standalone Rust app (`rust/app`) that does what the Go 9router gateway does, with the
original Go code kept only as reference. `rust/app` owns the entry point from day one (axum front
door for HTTP, clap binaries for the other entry points); everything not yet ported falls back to
the Go original via proxy/kill switch. When the fallback rate is zero and every unit passes `gate`,
run `app lift` to remove the fallback and ship a pure Rust binary.

This file is the plan and the claims; it points to the other files, never copies them.
Generated state: `rust-anything map --write` -> `rust/PORT_MAP.md` (measured). Compare at each
checkpoint.

## ADR index (rust/DECISIONS.md)
ADR-001 shape: all 6 entry points into rust/app, axum front door
ADR-002 scope: all features; Svelte UI (web/) stays outside Rust, embedded
ADR-003 equivalence: equal after normalizing; floats within 1e-9
ADR-004 compatibility: config/env/SQLite-DB/wire/CLI flags/HTTP routes+status identical
ADR-005 deps+runtime: tokio + axum; hand-wired DI (no fx clone)
ADR-006 targets: Linux x86_64 + aarch64, MSRV 1.84, single static binary, no C deps
ADR-007 checkpoints: go-ahead after every wave
ADR-008 cutover: beside-original, then lift, then switch traffic

## Unit plan (wave, unit, lines, state, bridge, readiness)
All states start `open`; a session claims one unit (`claimed by <session> <date>`), ports it,
marks it `done` with gate + replay results.

Wave 0 (no unit imports a later unit; port lowest first):
- internal/analyticsrange/ 132 done (cargo-verified; bridge blocked on `time.Time` from another package, hand-crafted as `rust/analyticsrange-rs` — Window/Resolve/since/bounded/count ported, chrono dep; `Resolve` logic fully covered by tests mirroring range_test.go) cgo 93
- internal/audit/ 1289 open cgo 0
- internal/browsercookies/ 1374 open cgo 0
- internal/browserlogin/ 687 open cgo 33
- internal/changelogfrag/ 331 open cgo 65
- internal/combos/smart/ 744 open cgo 53
- internal/connlimit/ 342 open cgo 55
- internal/constants/ 127 done (cargo-verified; bridge refused, no functions; hand-crafted as `rust/constants-rs` — consts + HttpTransportConfig + defaults ported; Go `Configure`/`NewTransport` on `*http.Transport` map to reqwest client-builder config in `rust/app`, not yet wired) cgo 77
- internal/customjson/ 494 open cgo 34
- internal/errclass/ 619 open cgo 65
- internal/fastjson/ 37 done (cargo-verified; bridge blocked on Go `any`+`io.Reader`, hand-crafted as `rust/fastjson-rs` serde_json lib) cgo 100
- internal/fetchgate/ 83 open cgo 87
- internal/integration/ 18 done (cargo-verified; bridge refused, no functions — only doc.go; hand-crafted as `rust/integration-rs`) cgo 100
- internal/integration/bootfx/ 15 done (cargo-verified; bridge refused, no functions — test-only TestMain harness; hand-crafted as `rust/bootfx-rs`, no runtime code) cgo 100
- internal/jsonfence/ 251 open cgo 56
- internal/log/ 378 open cgo 48
- internal/mitm/handlers/ 397 open cgo 49
- internal/models/ 361 open cgo 46
- internal/netguard/ 82 done (cargo-verified; bridge blocked on `net.IP` from another package, hand-crafted as `rust/netguard-rs` — parse_ip/is_private_or_reserved + cidrBlocks table ported, pure std) cgo 82
- internal/observ/ 355 open cgo 56
- internal/pricing/ 735 open cgo 49
- internal/proc/ 352 open cgo 47
- internal/promptcache/ 324 open cgo 58
- internal/proxy/aws/ 547 open cgo 28
- internal/proxy/executor/mcodelease/ 679 open cgo 21
- internal/resilience/ 1031 open cgo 5
- internal/shutdown/ 113 done (full rust-anything flow: gate passed, 5/5 diff, shadow written) cgo 81
- internal/testnet/ 52 done by session-2026-11-22 gate-passed cgo 94   [test-only helper, no HTTP route → not wired into front door; fallback rate unchanged]
- internal/tokensaver/ 3049 open cgo 0
- internal/tracing/ 123 open cgo 78
- tools/sitegen/ 2309 open cgo 0
- web/ 305 open cgo 70

Wave 1:
- cmd/changelog-merge/ 46 open cgo 100
- internal/config/ 263 open cgo 64
- internal/handlerutil/ 890 open cgo 3
- internal/mitm/ 675 open cgo 25
- internal/service/ 1116 open cgo 53
- internal/uptime/ 212 open cgo 54
- internal/vault/ 299 open cgo 62

Wave 2:
- internal/budget/ 297 open cgo 83
- internal/keikey/ 271 open cgo 48
- internal/providers/ 12233 open cgo 0
- internal/tunnel/ 3099 open cgo 0

Wave 3:
- internal/clientconfig/ 5490 open cgo 0
- internal/combopresets/ 140 open cgo 95
- internal/thinking/ 1865 open cgo 0

Wave 4:
- internal/db/ 13618 open cgo 0   [blocked until SQLite crate (no C) chosen — ADR-006]

Wave 5:
- internal/auth/ 719 open cgo 26
- internal/dbtest/ 112 open cgo 99
- internal/handlers/batches/ 337 open cgo 56
- internal/modelhealth/ 846 open cgo 21
- internal/policy/ 145 open cgo 77
- internal/translator/ 15179 open cgo 0

Wave 6:
- internal/daemon/ 682 open cgo 46
- internal/detection/ 284 open cgo 75
- internal/handlers/sso/ 1353 open cgo 0
- internal/middleware/ 2046 open cgo 0
- internal/proxy/ 5168 open cgo 0
- internal/semanticcache/ 960 open cgo 2
- internal/usagetracker/ 761 open cgo 0

Wave 7:
- benchmark/ 734 open cgo 23
- internal/codexquota/ 929 open cgo 21
- internal/guardrails/ 1555 open cgo 0
- internal/headroom/ 2575 open cgo 0
- internal/tray/ 438 open cgo 69
- internal/updater/ 964 open cgo 12

Wave 8:
- cmd/9router-tray/ 348 open cgo 47
- internal/handlers/shared/ 695 open cgo 32
- internal/proxy/executor/ 34475 open cgo 0
- internal/proxy/oauth/ 1555 open cgo 0

Wave 9:
- internal/handlers/chat/ 22167 open cgo 0
- internal/handlers/oauth/ 8709 open cgo 0

Wave 10:
- internal/handlers/media/ 13440 open cgo 0
- internal/schedule/ 1197 open cgo 0

Wave 11:
- internal/handlers/dashboard/ 25434 open cgo 0

Wave 12:
- internal/handlers/ 3930 open cgo 0

Wave 13:
- internal/app/ 839 open cgo 26

Wave 14:
- cmd/9router-go/ 897 open cgo 33

## Open questions
- SQLite crate with no C compiler: research in rust/research/modernc.org-sqlite.md (in flight).
  If none exists, internal/db/ (wave 4) stays blocked and the DB unit keeps falling back to Go.
- Which chi middleware / routes need re-recording because axum routing differs (ADR-005).
- `order`/`lines` noise cases in recorded output: confirm with user whether order is contract
  (ADR-003).

## Out of scope
- Rewriting the Svelte 5 dashboard in Rust (ADR-002).
- Packaging/release (done after lift, per user).
- Windows/macOS targets until Linux targets are solid (ADR-006).

## Linked files
rust/DECISIONS.md — ADRs (source of truth for decisions)
rust/GLOSSARY.md — domain terms -> Rust names
rust/HANDOFF.md — written after each checkpoint for the next session
rust/research/<topic>.md — dependency + risk research (primary sources only)
rust/app-cases/ — recorded CLI/HTTP cases (trycmd)
rust/app/ — the front door crate (built by `app front-door`)
