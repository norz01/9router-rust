//! The Rust port of `internal/netguard/`: `parse_ip` and `is_private_or_reserved`.
//!
//! Mirrors the Go package `internal/netguard/`:
//!   - `parse_ip(host)` strips an IPv6 zone (`%iface`) before parsing, so a
//!     zoned literal like "::1%lo" is not mistaken for "not an address".
//!   - `is_private_or_reserved(ip)` answers loopback/private/link-local/
//!     multicast/unspecified plus the extra CIDR blocks Go's `net.IP` methods
//!     do not cover (RFC6598 CGNAT, Oracle metadata, RFC2544 benchmark,
//!     6to4 anycast, documentation ranges).
//!
//! `net.IP` is `[]byte` and can be `nil` in Go; Rust's `std::net::IpAddr` is
//! non-nullable, so the nil → `false` branch of the Go original is enforced by
//! the type system (a caller can never pass an absent address).

use std::net::IpAddr;
use std::sync::OnceLock;

// ParseIP answers the address a literal host names, ignoring an IPv6 zone.
//
// `IpAddr::from_str` errors on a zoned literal ("::1%lo"), so callers must
// strip the zone first — exactly what this helper does, used instead of a
// bare parse on a host taken from a URL or dial address.
pub fn parse_ip(host: &str) -> Option<IpAddr> {
    let host = match host.find('%') {
        Some(i) => &host[..i],
        None => host,
    };
    host.parse::<IpAddr>().ok()
}

// cidrBlocks are the ranges Go's `net.IP` methods do not cover. Specs mirror
// the Go `cidrBlocks` table verbatim.
const CIDR_SPECS: &[(&str, u8)] = &[
    ("100.64.0.0", 10), // RFC6598 carrier-grade NAT; 100.100.100.200 is Alibaba's metadata service
    ("192.0.0.0", 24),  // RFC6890 IETF protocol assignments, incl. Oracle's 192.0.0.192 metadata
    ("198.18.0.0", 15), // RFC2544 benchmark range, used by some VM host-internal networks
    ("192.88.99.0", 24), // deprecated 6to4 relay anycast
    ("198.51.100.0", 24), // RFC5737 documentation ranges: never a real image host
    ("203.0.113.0", 24),
    ("::", 128),         // unspecified
    ("64:ff9b:1::", 48), // RFC8215 local-use NAT64
    ("2001:db8::", 32),  // documentation
];

struct Cidr {
    network: IpAddr,
    prefix: u8,
}

impl Cidr {
    fn contains(&self, ip: IpAddr) -> bool {
        match (self.network, ip) {
            (IpAddr::V4(n), IpAddr::V4(i)) => {
                let mask: u32 = if self.prefix == 0 {
                    0
                } else {
                    !0u32 << (32 - self.prefix)
                };
                u32::from(n) & mask == u32::from(i) & mask
            }
            (IpAddr::V6(n), IpAddr::V6(i)) => {
                let mask: u128 = if self.prefix == 0 {
                    0
                } else {
                    !0u128 << (128 - self.prefix)
                };
                u128::from(n) & mask == u128::from(i) & mask
            }
            _ => false,
        }
    }
}

fn cidr_blocks() -> &'static [Cidr] {
    static BLOCKS: OnceLock<Vec<Cidr>> = OnceLock::new();
    BLOCKS.get_or_init(|| {
        CIDR_SPECS
            .iter()
            .filter_map(|(spec, prefix)| {
                let network = spec.parse::<IpAddr>().ok()?;
                Some(Cidr {
                    network,
                    prefix: *prefix,
                })
            })
            .collect()
    })
}

// IsPrivateOrReserved reports whether an address belongs to loopback, private,
// link-local, cloud-metadata, carrier-grade-NAT, documentation or benchmark
// space — any destination an outbound request has no business reaching unless
// the operator asked for it by name.
pub fn is_private_or_reserved(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip4) => {
            if ip4.is_loopback()
                || ip4.is_private()
                || ip4.is_link_local()
                || ip4.is_multicast()
                || ip4.is_unspecified()
                || ip4.octets()[0] == 0
            {
                return true;
            }
        }
        IpAddr::V6(ip6) => {
            if ip6.is_loopback()
                || ip6.is_unique_local()
                || ip6.is_unicast_link_local()
                || ip6.is_multicast()
                || ip6.is_unspecified()
            {
                return true;
            }
        }
    }
    cidr_blocks().iter().any(|block| block.contains(ip))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> IpAddr {
        s.parse::<IpAddr>().expect("bad fixture")
    }

    #[test]
    fn blocks_private_and_reserved() {
        let blocked = [
            "127.0.0.1",       // loopback
            "10.1.2.3",        // RFC1918
            "172.16.0.1",      // RFC1918
            "192.168.1.1",     // RFC1918
            "169.254.169.254", // AWS/GCP metadata
            "0.0.0.0",         // unspecified
            "224.0.0.1",       // multicast
            "100.100.100.200", // Alibaba Cloud metadata (inside RFC6598 CGNAT)
            "100.64.0.1",      // RFC6598 CGNAT
            "192.0.0.192",     // Oracle Cloud metadata
            "198.18.0.1",      // RFC2544 benchmark
            "::1",             // IPv6 loopback
            "fe80::1",         // IPv6 link-local
            "2001:db8::1",     // documentation
        ];
        for s in blocked {
            assert!(is_private_or_reserved(parse(s)), "{s} must be blocked",);
        }
    }

    #[test]
    fn allows_public_addresses() {
        let allowed = [
            "1.1.1.1",
            "8.8.8.8",
            "93.184.216.34",  // example.com
            "100.63.255.255", // just below the CGNAT block
            "100.128.0.1",    // just above it
            "2606:4700::1111",
        ];
        for s in allowed {
            assert!(
                !is_private_or_reserved(parse(s)),
                "{s} must be reachable, the guard blocked it",
            );
        }
    }

    #[test]
    fn parse_ip_strips_zone() {
        let cases = [
            ("127.0.0.1", "127.0.0.1"),
            ("::1", "::1"),
            ("::1%lo", "::1"),
            ("fd00:ec2::254%eth0", "fd00:ec2::254"),
            ("::1%", "::1"),
        ];
        for (host, want) in cases {
            let ip = parse_ip(host).expect("parse_ip must not return None for a zoned literal");
            assert_eq!(ip.to_string(), want, "ParseIP({host})");
        }
    }

    #[test]
    fn parse_ip_rejects_non_literal() {
        for host in ["example.com", "", "not-an-ip"] {
            assert!(
                parse_ip(host).is_none(),
                "ParseIP({host}) = {:?}; a hostname is not a literal address",
                parse_ip(host),
            );
        }
    }

    #[test]
    fn zoned_private_keeps_verdict() {
        for host in ["::1%lo", "fd00:ec2::254%eth0", "fe80::1%eth0"] {
            let ip = parse_ip(host).expect("parse_ip");
            assert!(
                is_private_or_reserved(ip),
                "{host} parsed to {ip}, which the guard does not treat as private",
            );
        }
        let ip = parse_ip("2606:4700::1111%eth0").expect("parse_ip");
        assert!(
            !is_private_or_reserved(ip),
            "a public zoned address must stay reachable, got {ip}",
        );
    }

    #[test]
    fn cg_nat_boundary() {
        // 100.64.0.0/10 covers 100.64.0.0 .. 100.127.255.255.
        assert!(is_private_or_reserved(parse("100.64.0.0")));
        assert!(is_private_or_reserved(parse("100.127.255.255")));
        assert!(!is_private_or_reserved(parse("100.63.255.255")));
        assert!(!is_private_or_reserved(parse("100.128.0.0")));
    }
}
