//! The Rust port of `internal/constants/`: application-wide constants and the default
//! HTTP transport config.
//!
//! The values here are load-bearing: the default User-Agent is required by upstream WAFs
//! (Cloudflare fronts Nous Research's inference API and 403s the Go default), and the
//! transport timeouts/pool sizes govern upstream reverse-proxying and streaming.
//!
//! `internal/constants/` exports no functions — only consts, a struct and a package-level
//! variable — so `rust-anything bridge` cannot scaffold it; this crate is hand-crafted and
//! verified by `cargo test` asserting each value matches the Go original.

#![forbid(unsafe_code)]
#![expect(
    non_upper_case_globals,
    reason = "constant names mirror the Go source for traceability; GLOSSARY.md records the mapping"
)]

use std::time::Duration;

// HTTP Content Types
pub const ContentTypeJSON: &str = "application/json";
pub const ContentTypeEventStream: &str = "text/event-stream";

// HTTP Headers
pub const HeaderAuthorization: &str = "Authorization";
pub const HeaderContentType: &str = "Content-Type";
pub const HeaderCacheControl: &str = "Cache-Control";
pub const HeaderConnection: &str = "Connection";
pub const HeaderAccept: &str = "Accept";
pub const HeaderXApiKey: &str = "X-API-Key";
pub const HeaderUserAgent: &str = "User-Agent";

// Auth token prefix for header value (includes trailing space)
pub const AuthPrefixBearer: &str = "Bearer ";

// Default outbound client identity (load-bearing against upstream WAFs)
pub const DefaultClientUserAgent: &str = "node";

// Auth scheme names (used in ProviderConfig.AuthScheme)
pub const AuthSchemeBearer: &str = "bearer";
pub const AuthSchemeRaw: &str = "raw";

// Cache / Connection Directives
pub const CacheNoCache: &str = "no-cache";
pub const ConnKeepAlive: &str = "keep-alive";

// Default File Permissions
pub const FilePermDir: u32 = 0o755;
pub const FilePermFile: u32 = 0o644;
pub const FilePermKey: u32 = 0o600;

// SSE Format
pub const SSEFormat: &str = "data: %s\n\n";

// Lock Durations (seconds)
pub const DefaultLockDuration429: u64 = 60;
pub const DefaultLockDuration401: u64 = 120;

// Truncation Limits
pub const MaxResponseContentLen: usize = 10_000;
pub const MaxMessageContentLen: usize = 500;
pub const MaxLoggedMessages: usize = 20;

// Upstream Body Limits
pub const MaxUpstreamBodyBytes: usize = 10 << 20; // 10 MB cap for upstream response reads
pub const UpstreamErrLimit: usize = 1 << 20; // 1 MB cap for upstream error body reads

// Buffer Sizes
pub const StreamReadBuffer: usize = 4096; // 4 KB read buffer for streaming

// HTTPTransportConfig encapsulates connection pooling and timeout configurations for
// upstream HTTP transports in reverse proxying and streaming workloads.
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub max_idle_conns: i64,
    pub max_idle_conns_per_host: i64,
    pub idle_conn_timeout: Duration,
    pub tls_handshake_timeout: Duration,
    pub expect_continue_timeout: Duration,
    pub response_header_timeout: Duration,
}

// DefaultHTTPTransportConfig defines the high-throughput production defaults for upstream
// LLM reverse-proxy connections.
pub const DefaultHttpTransportConfig: HttpTransportConfig = HttpTransportConfig {
    max_idle_conns: 256,
    max_idle_conns_per_host: 128,
    idle_conn_timeout: Duration::from_secs(90),
    tls_handshake_timeout: Duration::from_secs(10),
    expect_continue_timeout: Duration::from_secs(1),
    response_header_timeout: Duration::from_secs(120),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_types_match_go() {
        assert_eq!(ContentTypeJSON, "application/json");
        assert_eq!(ContentTypeEventStream, "text/event-stream");
    }

    #[test]
    fn headers_match_go() {
        assert_eq!(HeaderAuthorization, "Authorization");
        assert_eq!(HeaderContentType, "Content-Type");
        assert_eq!(HeaderCacheControl, "Cache-Control");
        assert_eq!(HeaderConnection, "Connection");
        assert_eq!(HeaderAccept, "Accept");
        assert_eq!(HeaderXApiKey, "X-API-Key");
        assert_eq!(HeaderUserAgent, "User-Agent");
    }

    #[test]
    fn auth_values_match_go() {
        assert_eq!(AuthPrefixBearer, "Bearer ");
        assert_eq!(DefaultClientUserAgent, "node");
        assert_eq!(AuthSchemeBearer, "bearer");
        assert_eq!(AuthSchemeRaw, "raw");
    }

    #[test]
    fn cache_and_connection_match_go() {
        assert_eq!(CacheNoCache, "no-cache");
        assert_eq!(ConnKeepAlive, "keep-alive");
    }

    #[test]
    fn file_permissions_match_go() {
        assert_eq!(FilePermDir, 0o755);
        assert_eq!(FilePermFile, 0o644);
        assert_eq!(FilePermKey, 0o600);
    }

    #[test]
    fn sse_format_matches_go() {
        assert_eq!(SSEFormat, "data: %s\n\n");
    }

    #[test]
    fn lock_durations_match_go() {
        assert_eq!(DefaultLockDuration429, 60);
        assert_eq!(DefaultLockDuration401, 120);
    }

    #[test]
    fn truncation_limits_match_go() {
        assert_eq!(MaxResponseContentLen, 10_000);
        assert_eq!(MaxMessageContentLen, 500);
        assert_eq!(MaxLoggedMessages, 20);
    }

    #[test]
    fn upstream_body_limits_match_go() {
        assert_eq!(MaxUpstreamBodyBytes, 10 << 20);
        assert_eq!(UpstreamErrLimit, 1 << 20);
    }

    #[test]
    fn buffer_sizes_match_go() {
        assert_eq!(StreamReadBuffer, 4096);
    }

    #[test]
    fn default_transport_config_matches_go() {
        let c = DefaultHttpTransportConfig;
        assert_eq!(c.max_idle_conns, 256);
        assert_eq!(c.max_idle_conns_per_host, 128);
        assert_eq!(c.idle_conn_timeout, Duration::from_secs(90));
        assert_eq!(c.tls_handshake_timeout, Duration::from_secs(10));
        assert_eq!(c.expect_continue_timeout, Duration::from_secs(1));
        assert_eq!(c.response_header_timeout, Duration::from_secs(120));
    }
}
