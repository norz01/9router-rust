//! The port of `internal/integration/bootfx/`.
//!
//! The Go original has no exported library functions: it is an `integration`-tagged test
//! package whose `TestMain` boots the whole `fx` application graph and whose tests assert
//! end-to-end traffic and auth on the live server. There is nothing to port into a library
//! — the equivalent integration harness belongs in `rust/app` (axum front door, hand-wired
//! DI), not here.
//!
//! This module is intentionally empty of runtime code. It exists so the crate has a
//! `port` module matching the repo convention (`src/port.rs`) and so `cargo build`,
//! `cargo test`, `cargo clippy` and `cargo fmt --check` all pass.

// No runtime code: the unit is test-only infrastructure with no library exports.
