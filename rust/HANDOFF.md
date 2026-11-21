# HANDOFF.md — checkpoint after wave 0

Status: wave 0 complete. Fallback rate 100% (all 24 HTTP cases still proxied to the Go original;
expected — wave 0 is library/utility units with no HTTP routes or CLI commands; they become Rust
dependencies for later HTTP-handler units, not front-door routes).

## Where the port stands
- rust/DECISIONS.md — 8 ADRs (user-approved): shape, scope, equivalence (equal after normalizing,
  floats within 1e-9), compatibility (config/env/SQLite/wire/CLI/routes identical), tokio+axum +
  hand-wired DI, Linux x86_64+aarch64 MSRV 1.85 no-C-deps, checkpoint after every wave,
  beside-original then lift then switch.
- rust/GLOSSARY.md — domain terms -> Rust names.
- rust/MAP.md — unit plan with states; wave 0 all `done`.
- rust/app-cases/http/ — 24 recorded cases; original passes 24/24 (100%).
- rust/app/ — axum front door, proxies all 24 routes to Go on a free port; replay 24/24,
  fallback rate 100%.

## Wave 0 units done (7 of 7)
| Unit | Lines | How ported | Verification |
|------|-------|------------|--------------|
| internal/testnet/ | 52 | full rust-anything flow | gate passed, diff 1/1, shadow written (test-only helper, no route → not wired) |
| internal/shutdown/ | 113 | full rust-anything flow | gate passed, diff 5/5, shadow written |
| internal/fastjson/ | 37 | hand-crafted `rust/fastjson-rs` (bridge blocked: Go `any`+`io.Reader`) | cargo build+test+clippy `-D warnings`+fmt clean (5 tests mirror range/fastjson tests) |
| internal/constants/ | 127 | hand-crafted `rust/constants-rs` (bridge refused: no functions) | cargo build+test+clippy `-D warnings`+fmt clean (11 tests assert each const == Go) |
| internal/integration/ | 18 | hand-crafted `rust/integration-rs` (bridge refused: only doc.go) | cargo build+test+clippy+fmt clean |
| internal/integration/bootfx/ | 15 | hand-crafted `rust/bootfx-rs` (bridge refused: no exports) | cargo build+test+clippy+fmt clean |
| internal/analyticsrange/ | 132 | hand-crafted `rust/analyticsrange-rs` (bridge blocked: `time.Time` from another package) | cargo build+test+clippy `-D warnings`+fmt clean (5 tests mirror range_test.go) |
| internal/netguard/ | 82 | hand-crafted `rust/netguard-rs` (bridge blocked: `net.IP` from another package) | cargo build+test+clippy `-D warnings`+fmt clean (6 tests mirror netguard_test.go) |

## What's next (wave 1)
- internal/config/ (263, readiness 64) — viper/TOML/YAML config; ADR-004 says keep format identical.
- internal/service/ (1116, readiness 53) — fx-based service wiring → hand-wired DI (ADR-005).
- internal/handlerutil/ (890, readiness 3), internal/mitm/ (675, 25), internal/vault/ (299, 62),
  internal/uptime/ (212, 54), internal/changelog-merge/ (46, 100).

## Blocked / open
- internal/db/ (wave 4, 13618 lines) blocked until a no-C SQLite crate is confirmed (ADR-006:
  rust-version 1.84 → 1.85 because edition 2024 needs 1.85+; MSRV should be updated in DECISIONS.md).
  SQLite research not yet written back (subagent didn't complete the file).
- `app wire` requires `--route` even for library units; wave-0 library units aren't wired to HTTP
  (correct — they have no routes). HTTP-handling units come in later waves (chat, handlers,
  middleware, proxy).

## Files to read next
rust/MAP.md (plan + claims), rust/DECISIONS.md (ADRs), rust/GLOSSARY.md (terms), rust/app-cases/
(recorded cases), rust/app/ (front door).

## Skill to load next
This skill (whole-app port): `skill rust-anything` / SKILL.md — steps 1-5 per unit, then wire +
replay, then lift.

## Last checkpoint results
- `rust-anything app replay` against front door: 24/24 passed, fallback rate 100%.
- gate passed for: testnet (1/1), shutdown (5/5).
- cargo-verified (build/test/clippy/fmt): fastjson-rs, constants-rs, integration-rs, bootfx-rs,
  analyticsrange-rs, netguard-rs.
