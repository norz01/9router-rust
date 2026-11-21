//! The Rust port of `internal/analyticsrange/`: resolves the time window an
//! analytics read covers.
//!
//! Mirrors the Go package faithfully:
//!   - `Window` holds an inclusive lower-bound cutoff plus an hour width;
//!   - `Resolve` maps a period string onto a window, falling back to 24h on
//!     unrecognised input rather than erroring;
//!   - `today` is anchored at local midnight, not a sliding 24 hours.

pub mod port;

pub use port::{Window, resolve};
