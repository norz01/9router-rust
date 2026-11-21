//! The Rust port of `internal/fastjson/`.

pub mod port;

pub use port::{marshal, unmarshal, unmarshal_read};
