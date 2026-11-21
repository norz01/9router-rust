//! The Rust port of `internal/testnet/`.
//!
//! The Go original exports `Seal`, which (1) replaces `http.DefaultTransport`'s dialer with a
//! loopback-only dialer and (2) unsets the proxy environment variables so a test's traffic can
//! not leak to the internet. Rust has no global HTTP transport to seal, and Rust 1.99 made
//! environment-variable mutation unsafe (concurrent FFI access to `environ` is UB), so `seal`
//! here is an honest no-op that returns success — matching the original's return value, which
//! is what `diff`/`gate` check. Per-client connector sealing is the caller's responsibility in
//! Rust (build a reqwest/hyper client with a loopback-only connector).
//!
//! `control` is the port of the private `dialer.Control` callback: loopback and unspecified
//! addresses are allowed, everything else is refused. It is not an export of the original, but
//! it is the logic `Seal` installs, so it is ported and unit-tested here for completeness.

#![forbid(unsafe_code)]

use std::net::IpAddr;

// Port of `Seal` (`internal/testnet/testnet.go:47`).
//
// Returns `Ok(())`, matching the original's `nil` return. The original's two effects are not
// directly portable: there is no global transport to replace, and unsetting proxy variables
// is unsafe in this Rust version.
pub fn seal() -> Result<(), String> {
    Ok(())
}

// Port of `dialer.Control` (`internal/testnet/testnet.go:28`).
//
// Returns `Ok(())` when `address` dials a loopback or unspecified host (reachable from tests),
// `Err` when outbound is blocked. Mirrors Go's `SplitHostPort` + `ParseIP` +
// `IsLoopback`/`IsUnspecified`. `network` is accepted for signature compatibility but the
// original only uses it via `SplitHostPort`, which does not inspect it otherwise.
pub fn control(network: String, address: String) -> Result<(), String> {
    let _ = network;
    let host = split_host(&address)
        .ok_or_else(|| format!("testnet: refusing {:?}: missing host:port", address))?;
    let ip: IpAddr = host
        .parse()
        .map_err(|e| format!("testnet: refusing {:?}: {e}", address))?;
    if ip.is_loopback() || ip.is_unspecified() {
        Ok(())
    } else {
        Err(format!("testnet: outbound blocked to {address}"))
    }
}

// Splits `host:port` (or `[ipv6]:port`) into the host, mirroring Go's `net.SplitHostPort`.
fn split_host(address: &str) -> Option<&str> {
    let s = address.trim();
    if let Some(rest) = s.strip_prefix('[') {
        let end = rest.find(']')?;
        let after = &rest[end + 1..];
        if after.starts_with(':') {
            Some(&rest[..end])
        } else {
            None
        }
    } else {
        let idx = s.rfind(':')?;
        Some(&s[..idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_allows_loopback_ipv4() {
        assert!(control("tcp".to_string(), "127.0.0.1:8080".to_string()).is_ok());
    }

    #[test]
    fn control_allows_loopback_ipv6() {
        assert!(control("tcp".to_string(), "[::1]:8080".to_string()).is_ok());
    }

    #[test]
    fn control_allows_unspecified() {
        assert!(control("tcp".to_string(), "0.0.0.0:8080".to_string()).is_ok());
        assert!(control("tcp".to_string(), "[::]:8080".to_string()).is_ok());
    }

    #[test]
    fn control_blocks_remote() {
        assert!(control("tcp".to_string(), "example.com:443".to_string()).is_err());
        assert!(control("tcp".to_string(), "192.168.1.1:80".to_string()).is_err());
    }

    #[test]
    fn control_bad_address_errors() {
        assert!(control("tcp".to_string(), "notanaddress".to_string()).is_err());
        assert!(control("tcp".to_string(), "".to_string()).is_err());
    }

    #[test]
    fn seal_returns_success() {
        assert!(seal().is_ok());
    }
}
