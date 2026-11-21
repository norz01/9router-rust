//! The Rust port of `internal/netguard/`: classification of addresses that must
//! never be reached by an outbound request the gateway did not explicitly choose.
//!
//! One predicate, one place to fix it. The Go original's `net.IP`-based
//! predicates miss cloud/carrier ranges (RFC6598 CGNAT, Oracle metadata,
//! RFC2544 benchmark, documentation ranges) that this port covers explicitly,
//! mirroring the Go `cidrBlocks` table.

#![forbid(unsafe_code)]

pub mod port;

pub use port::{is_private_or_reserved, parse_ip};
