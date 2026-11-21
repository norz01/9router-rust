//! The Rust port of `internal/analyticsrange/`: resolves the time window an
//! analytics read covers.
//!
//! Mirrors the Go package faithfully:
//!   - `Window` holds an inclusive lower-bound cutoff plus an hour width;
//!   - `Resolve` maps a period string onto a window, falling back to 24h on
//!     unrecognised input rather than erroring;
//!   - `today` is anchored at midnight (of `now`, in UTC) rather than a
//!     sliding 24 hours, so "Today" and "Last 24 hours" stay distinguishable.
//!
//! `now` is a parameter rather than read from the clock so a caller resolving
//! several ranges for one response gets one instant.

use chrono::{DateTime, Datelike, TimeZone, Utc};

// maxWindowHours bounds every window, including the unbounded one. usageHistory
// holds one row per request. When an operator has configured a retention
// window, `all` spans as far back as rows are kept; when retention is off (the
// default), it is open-ended. The ceiling stops a typo'd period from asking for
// a scan that cannot finish. Ten years is past any ledger this build has written.
// The bound also keeps the arithmetic below inside time.Duration: a ceiling near
// the 292-year limit makes now.Sub(cutoff) overflow rather than clamp, and the
// overflow reads back as a window far shorter than the one that was asked for.
const MAX_WINDOW_HOURS: i64 = 10 * 365 * 24;

// Window is a resolved time window. The zero value is the unbounded window, so
// a caller that never resolves one reads the whole ledger — which is what these
// analytics did before any window was plumbed through.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Window {
    // cutoff is the inclusive lower bound of the window. None means unbounded.
    cutoff: Option<DateTime<Utc>>,
    // hours is the window's width in hours, rounded up. Zero for unbounded.
    hours: i64,
}

impl Window {
    // Cutoff returns the inclusive lower bound, or None for an unbounded window.
    pub fn cutoff(&self) -> Option<DateTime<Utc>> {
        self.cutoff
    }

    // Hours returns the window's width, rounded up to the hour. Trend charts
    // bucket by hour, so a sub-hour window still yields at most one bucket and
    // rounding keeps every caller from having to special-case zero.
    pub fn hours(&self) -> i64 {
        self.hours
    }

    // Unbounded reports whether the window has no lower bound.
    pub fn unbounded(&self) -> bool {
        self.cutoff.is_none()
    }

    // Label names the window the way the dashboard's picker does, so a response
    // can say which period produced it. An unrecognised period is reported
    // verbatim rather than as its fallback, which is the only way a caller could
    // notice the server answered a different window than the one it asked for.
    pub fn label(&self, requested: &str) -> String {
        if self.unbounded() {
            return "all".to_string();
        }
        requested.to_string()
    }
}

// Resolve maps a period parameter onto a window.
//
// Accepted: today, 24h, 7d, 30d, 60d, all, and the <n>d / <n>h forms. `today`
// is anchored at midnight rather than measured back 24 hours, so "Today" and
// "Last 24 hours" stay distinguishable in the label the operator picked.
pub fn resolve(raw: &str, now: DateTime<Utc>) -> Window {
    let raw = raw.trim().to_lowercase();

    match raw.as_str() {
        "" | "24h" => return since(now, 24),
        "today" => {
            let d = now.date_naive();
            let midnight = Utc
                .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
                .single()
                .unwrap();
            return bounded(now, midnight);
        }
        "7d" => return since(now, 7 * 24),
        "30d" => return since(now, 30 * 24),
        "60d" => return since(now, 60 * 24),
        "all" => return Window::default(),
        _ => {}
    }

    if let Some(n) = count(&raw, 'd') {
        return since(now, n * 24);
    }
    if let Some(n) = count(&raw, 'h') {
        return since(now, n);
    }

    // Unrecognised input falls back to the 24h default rather than erroring:
    // the SPA can be newer than the gateway it talks to.
    since(now, 24)
}

// since builds a window ending at now.
fn since(now: DateTime<Utc>, width_hours: i64) -> Window {
    bounded(now, now - chrono::Duration::hours(width_hours))
}

// bounded normalises a cutoff and measures the width against now. Rounding the
// cutoff down to the second is not cosmetic: the ledger stores RFC3339 with
// second precision, so a sub-second cutoff compares lexicographically against
// a row written in the same second and excludes it.
fn bounded(now: DateTime<Utc>, cutoff: DateTime<Utc>) -> Window {
    let cutoff = truncate_to_second(cutoff);
    let diff = now - cutoff;
    let mut hours = diff.num_hours() + 1;
    if hours > MAX_WINDOW_HOURS {
        hours = MAX_WINDOW_HOURS;
    }
    Window {
        cutoff: Some(cutoff),
        hours,
    }
}

// count parses the "<n><unit>" shorthand, accepting only a positive count with
// no sign and no decimal part. A count above the ceiling is clamped rather than
// rejected: a caller asking for a 99999h window wants "as far back as this
// goes", and an error would read as a broken page.
fn count(raw: &str, unit: char) -> Option<i64> {
    if raw.len() < 2 || !raw.ends_with(unit) {
        return None;
    }
    let digits = &raw[..raw.len() - 1];
    let n: i64 = match digits.parse() {
        Ok(n) => n,
        Err(_) => return None,
    };
    if n <= 0 {
        return None;
    }
    Some(n.min(MAX_WINDOW_HOURS))
}

fn truncate_to_second(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt - chrono::Duration::nanoseconds(dt.timestamp_subsec_nanos() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 15, 30, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn resolve_matches_go_table() {
        let now = test_now();
        let cases: &[(&str, DateTime<Utc>, i64, bool)] = &[
            ("", now - chrono::Duration::hours(24), 25, false),
            ("24h", now - chrono::Duration::hours(24), 25, false),
            (
                "7d",
                now - chrono::Duration::hours(7 * 24),
                7 * 24 + 1,
                false,
            ),
            (
                "30d",
                now - chrono::Duration::hours(30 * 24),
                30 * 24 + 1,
                false,
            ),
            (
                "60d",
                now - chrono::Duration::hours(60 * 24),
                60 * 24 + 1,
                false,
            ),
            ("12h", now - chrono::Duration::hours(12), 13, false),
            (
                "14d",
                now - chrono::Duration::hours(14 * 24),
                14 * 24 + 1,
                false,
            ),
            ("last-tuesday", now - chrono::Duration::hours(24), 25, false),
            ("0d", now - chrono::Duration::hours(24), 25, false),
            ("-3d", now - chrono::Duration::hours(24), 25, false),
            ("1.5d", now - chrono::Duration::hours(24), 25, false),
        ];
        for &case in cases {
            let (raw, want_cutoff, want_hours, unbounded) = case;
            let got = resolve(raw, now);
            assert_eq!(
                got.unbounded(),
                unbounded,
                "{raw}: unbounded = {}, want {}",
                got.unbounded(),
                unbounded
            );
            if unbounded {
                continue;
            }
            assert!(got.cutoff().is_some(), "{raw}: cutoff is none");
            assert_eq!(
                got.cutoff().unwrap(),
                want_cutoff,
                "{raw}: cutoff = {:?}, want {:?}",
                got.cutoff(),
                want_cutoff
            );
            assert_eq!(got.hours(), want_hours, "{raw}: hours");
        }
    }

    #[test]
    fn resolve_all_is_unbounded() {
        let now = test_now();
        let got = resolve("all", now);
        assert!(got.unbounded());
        assert!(got.cutoff().is_none());
        assert_eq!(got.hours(), 0);
        assert_eq!(got.label("all"), "all");
    }

    #[test]
    fn today_is_anchored_not_sliding() {
        let now = test_now();
        let today = resolve("today", now);
        let want = Utc
            .with_ymd_and_hms(2026, 10, 10, 0, 0, 0)
            .single()
            .unwrap();
        assert_eq!(today.cutoff(), Some(want));

        let sliding = resolve("24h", now);
        assert_ne!(today.cutoff(), sliding.cutoff());
    }

    #[test]
    fn resolve_truncates_to_stored_precision() {
        let now = Utc
            .with_ymd_and_hms(2026, 10, 10, 15, 30, 0)
            .single()
            .unwrap()
            + chrono::Duration::nanoseconds(987654321);
        let win = resolve("24h", now);
        assert_eq!(
            win.cutoff().unwrap().timestamp_subsec_nanos(),
            0,
            "cutoff carries sub-second nanos; the ledger stores whole seconds"
        );
    }

    #[test]
    fn resolve_clamps_absurd_windows() {
        let now = test_now();
        let win = resolve("99999h", now);
        assert!(
            !win.unbounded(),
            "a finite request must not resolve to the unbounded window"
        );
        assert_eq!(
            win.hours(),
            MAX_WINDOW_HOURS,
            "Hours() = {}, want the clamp at {}",
            win.hours(),
            MAX_WINDOW_HOURS
        );
    }
}
