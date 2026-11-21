//! The Rust port of `internal/integration/bootfx/`.
//!
//! The Go original is an `integration`-tagged test package: `TestMain` boots the entire
//! `fx` application graph (ConfigModule, DatabaseModule, HandlersModule, ServerModule) once
//! against a temp data dir and a free port, then the tests drive real HTTP traffic through
//! it to a fake upstream.
//!
//! This crate has no library exports: the package exports no functions, only `TestMain` and
//! test functions, so `rust-anything bridge` is blocked ("exports no functions"). This crate
//! is therefore hand-crafted. There is no Rust equivalent of `fx` (ADR-005: hand-wired DI,
//! no fx clone), and integration testing lives in `rust/app`, not here.
//!
//! This crate compiles cleanly and carries no runtime logic. `cargo test` passes with no
//! tests to run.

#![forbid(unsafe_code)]

pub mod port;
