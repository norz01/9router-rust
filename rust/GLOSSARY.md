# GLOSSARY.md — domain terms for the 9router-rust port

Each term: original meaning, Rust name (type/module/function) used in the port. Grows as units
bring new terms. Point to files; copy nothing here into other files.

| Term | Meaning in original (Go) | Rust name in port |
|------|--------------------------|-------------------|
| provider | An LLM upstream (antigravity, opencode, claude, codex, cline, gemini, xai, …) with its own auth, envelope, model catalog. Strictly isolated — never aliased into another provider. | `internal/providers/` registry; `Provider` type; executor per provider in `rust/app/src/units/<provider>/` or a shared `proxy/executor/` crate |
| connection | A stored provider + credentials + base URL used to reach an upstream. | `internal/db/` → Rust DB crate; `Connection` struct |
| combo | A named group of connections with model fallback / account rotation / strike-breaker quotas. | `internal/combos/` → `Combo` struct; combo expansion in Rust |
| proxy pool | A pool of upstream URLs/accounts rotated for a combo. | `ProxyPool` |
| model | A model ID (`ag/<model>`, `oc/<model>`, …) resolved via aliases + catalog. | `ResolveModelAlias`; `registry_models` |
| chat completion | `/v1/chat/completions` (OpenAI) or Messages (Anthropic) request routed through a combo with fallback. | `internal/handlers/chat/` → Rust chat handler |
| SSE stream | Text/event-stream response from an upstream LLM, forwarded/repaired (tool ID repair, cloaking). | streamed via `tokio` + `bytes`, `axum::response::Stream` |
| token saver / RTK | Prompt compression (RTK, Caveman, Ponytail system prompts). | `internal/tokensaver/` |
| usage history | Rows written to SQLite (`~/.9router/db/data.sqlite`) for accounting / quotas. | `internal/db/` `usageHistory` table |
| strike breaker | Quota handling that marks a connection/account struck when it fails. | `strike-breaker` logic in combo fallback |
| dashboard | Svelte 5 UI at `web/`, embedded, calls `GET /api/*` and `POST /api/*`. | kept as-is; served by Rust app |
| changelog fragment | `.changes/<pr>-<slug>.md`; `GET /api/changelog` assembles CHANGELOG.md + fragments. | `internal/changelogfrag/` → Rust |
| kill switch | `RUST_ANYTHING_UNIT_<NAME>=original` sends a ported unit back to the Go original until lift. | env var checked in `rust/app/src/units/<unit>.rs` glue |

Rules carried from AGENTS.md §3 (provider isolation) and §4 (Go engineering): no cross-provider
aliasing, no substring-based provider switching, stream don't buffer, one read per page (no N+1)
on SQLite, `lo`-style concise collection ops, `#[expect(lint, reason=…)]` not `#[allow]`.
