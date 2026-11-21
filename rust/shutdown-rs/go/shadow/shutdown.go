// Package shutdown runs the Go original and the Rust port of `internal/shutdown/` side by side. Generated
// by rust-anything: import it in place of 9router/proxy/internal/shutdown. RUST_ANYTHING_MODE=old|shadow|new.
package shutdown

import (
	"encoding/json"
	"log"
	"os"
	"reflect"
	"sync/atomic"

	orig "9router/proxy/internal/shutdown"
	port "rust-anything.local/shutdown_rs"
)

var mode = func() string {
	if os.Getenv("RUST_ANYTHING_UNIT_SHUTDOWN") == "original" {
		return "old"
	}
	if m := os.Getenv("RUST_ANYTHING_MODE"); m != "" {
		return m
	}
	return "shadow"
}()

// Calls and Mismatches count shadowed calls.
var Calls, Mismatches atomic.Int64

// convert moves a value between the original's and the wrapper's identical types.
func convert[T any](v any) T {
	var out T
	if text, err := json.Marshal(v); err == nil {
		_ = json.Unmarshal(text, &out)
	}
	return out
}

func compare(name string, expected, actual []any) {
	Calls.Add(1)
	if !reflect.DeepEqual(expected, actual) {
		if Mismatches.Add(1) <= 10 {
			log.Printf("[rust-anything] mismatch in %s: expected %v, got %v", name, expected, actual)
		}
	}
}

// Fired shadows the original `Fired`.
func Fired() bool {
	if mode == "new" {
		n0 := port.Fired()
		return n0
	}
	o0 := orig.Fired()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("Fired", []any{"no panic"}, []any{r})
				}
			}()
			n0 := port.Fired()
			compare("Fired", []any{o0}, []any{n0})
		}()
	}
	return o0
}

// RequestStop shadows the original `RequestStop`.
func RequestStop() {
	if mode == "new" {
		port.RequestStop()

	}
	orig.RequestStop()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("RequestStop", []any{"no panic"}, []any{r})
				}
			}()
			port.RequestStop()
			compare("RequestStop", []any{}, []any{})
		}()
	}
	return
}

// TestReset shadows the original `TestReset`.
func TestReset() {
	if mode == "new" {
		port.TestReset()

	}
	orig.TestReset()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("TestReset", []any{"no panic"}, []any{r})
				}
			}()
			port.TestReset()
			compare("TestReset", []any{}, []any{})
		}()
	}
	return
}

// RunAfterStop shadows the original `RunAfterStop`.
func RunAfterStop() {
	if mode == "new" {
		port.RunAfterStop()

	}
	orig.RunAfterStop()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("RunAfterStop", []any{"no panic"}, []any{r})
				}
			}()
			port.RunAfterStop()
			compare("RunAfterStop", []any{}, []any{})
		}()
	}
	return
}

// Cancel shadows the original `Cancel`.
func Cancel() {
	if mode == "new" {
		port.Cancel()

	}
	orig.Cancel()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("Cancel", []any{"no panic"}, []any{r})
				}
			}()
			port.Cancel()
			compare("Cancel", []any{}, []any{})
		}()
	}
	return
}

func errText(err error) any {
	if err == nil {
		return nil
	}
	return err.Error()
}
