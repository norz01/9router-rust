//! The Rust port of `internal/integration/`: feature-level integration suite
//! documentation.
//!
//! The Go unit exports no functions, types or constants —
//! `internal/integration/doc.go` is purely a package doc comment describing how
//! the integration test suite is organized: it boots the real HTTP stack
//! against a temporary database and a fake LLM upstream, is gated behind the
//! `integration` build tag, and the doc file is intentionally untagged so the
//! package always has at least one compilable Go file (`go build ./...` / `go
//! vet ./...` never hit a directory whose every file is excluded by the tag).
//!
//! Because there is nothing to port, this crate is hand-crafted (same
//! precedent as `internal/constants/`, which also exports no functions) and is
//! verified by `cargo build` + `cargo test` + `cargo clippy` + `cargo fmt`.
//! Nothing is wired into `rust/app`; the integration suite remains a Go-side
//! concern until a later unit ports the handlers it exercises.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds_cleanly() {
        // No functions to port; this test exists so `cargo test` exercises the
        // crate and confirms it builds and runs cleanly.
    }
}
