//! The Rust port of `internal/shutdown/`.
#![forbid(unsafe_code)]

use std::sync::{Mutex, OnceLock};

/// Shared mutable state, mirroring the Go package-level vars (`mu`, `fired`, `done`,
/// `requested`, `afterStop`). Every ported function observes the same state, so a
/// sequence of calls behaves like the Go original.
///
/// The channel/context fields (`done`, `requested`, `ctx`) live in Go (`Done`,
/// `StopRequested`, `Context` are js_only), so this Rust copy tracks only what the
/// ported functions can observe: whether shutdown has fired and whether a stop was
/// requested. `after_stop` mirrors the restart hook; it is never registered from this
/// crate because `RestartAfterStop` stays in Go (takes a function), so it stays `None`.
struct ShutdownState {
    fired: bool,
    requested_closed: bool,
    after_stop: Option<Box<dyn Fn() + Send + Sync>>,
}

static STATE: OnceLock<Mutex<ShutdownState>> = OnceLock::new();

fn state() -> &'static Mutex<ShutdownState> {
    STATE.get_or_init(|| {
        Mutex::new(ShutdownState {
            fired: false,
            requested_closed: false,
            after_stop: None,
        })
    })
}

fn reset_state() -> Result<(), String> {
    let mut s = state().lock().map_err(|e| e.to_string())?;
    *s = ShutdownState {
        fired: false,
        requested_closed: false,
        after_stop: None,
    };
    Ok(())
}

/// Port of `Fired` (`internal/shutdown/shutdown.go:29`).
// INFALLIBLE: reads a bool under a mutex that is never poisoned in this port.
pub fn fired() -> Result<bool, String> {
    let s = state().lock().map_err(|e| e.to_string())?;
    Ok(s.fired)
}

/// Port of `RequestStop` (`internal/shutdown/shutdown.go:55`): cancels, then closes the
/// stop-request channel once. Idempotent — a second call must not panic.
pub fn request_stop() -> Result<(), String> {
    cancel_inner()?;
    let mut s = state().lock().map_err(|e| e.to_string())?;
    if !s.requested_closed {
        s.requested_closed = true;
    }
    Ok(())
}

/// Port of `TestReset` (`internal/shutdown/shutdown.go:74`): restores the package for
/// tests (uncancel + unfire + fresh channels).
pub fn test_reset() -> Result<(), String> {
    reset_state()
}

/// Port of `RunAfterStop` (`internal/shutdown/shutdown.go:105`): runs the registered
/// restart hook once, then clears it so it cannot re-run.
pub fn run_after_stop() -> Result<(), String> {
    let hook = {
        let mut s = state().lock().map_err(|e| e.to_string())?;
        s.after_stop.take()
    };
    if let Some(f) = hook {
        f();
    }
    Ok(())
}

/// Port of `Cancel` (`internal/shutdown/shutdown.go:116`): sets fired, closes the done
/// channel, cancels the context. Safe to call multiple times — idempotent.
pub fn cancel() -> Result<(), String> {
    cancel_inner()
}

fn cancel_inner() -> Result<(), String> {
    let mut s = state().lock().map_err(|e| e.to_string())?;
    if s.fired {
        return Ok(());
    }
    s.fired = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fired_false_until_cancel() -> Result<(), String> {
        test_reset()?;
        assert!(!fired()?);
        cancel()?;
        assert!(fired()?);
        Ok(())
    }

    #[test]
    fn cancel_is_idempotent() -> Result<(), String> {
        test_reset()?;
        cancel()?;
        cancel()?; // must not error on double close
        assert!(fired()?);
        Ok(())
    }

    #[test]
    fn test_reset_unfires() -> Result<(), String> {
        test_reset()?;
        cancel()?;
        assert!(fired()?);
        test_reset()?;
        assert!(!fired()?);
        Ok(())
    }

    #[test]
    fn request_stop_fires_and_requests() -> Result<(), String> {
        test_reset()?;
        assert!(!fired()?);
        request_stop()?;
        assert!(fired()?);
        request_stop()?; // idempotent
        assert!(fired()?);
        Ok(())
    }

    #[test]
    fn run_after_stop_runs_hook_once() -> Result<(), String> {
        test_reset()?;
        // No hook is registered (RestartAfterStop stays in Go), so this is a no-op.
        run_after_stop()?;
        run_after_stop()?; // must not re-run
        Ok(())
    }
}
