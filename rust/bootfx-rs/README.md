# bootfx-rs

Rust port of `internal/integration/bootfx/`, hand-crafted (bridge blocked: the unit exports
no functions — it is an `integration`-tagged test package whose `TestMain` boots the whole
`fx` application graph and whose tests assert end-to-end traffic and auth on the live server).

## Functions

None. The Go original has no exported library functions, so there is nothing to port into a
Rust library. `rust-anything bridge` is blocked ("exports no functions"); this crate is
hand-crafted and verified by `cargo build`, `cargo test`, `cargo clippy` and `cargo fmt`.

The equivalent integration harness belongs in `rust/app` (axum front door, hand-wired DI per
ADR-005; there is no Rust equivalent of `fx`), not here.

## Loop

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
