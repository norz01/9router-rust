// Package testnet runs the Go original and the Rust port of `internal/testnet/` side by side. Generated
// by rust-anything: import it in place of 9router/proxy/internal/testnet. RUST_ANYTHING_MODE=old|shadow|new.
package testnet

import (
	"encoding/json"
	"log"
	"os"
	"reflect"
	"sync/atomic"

	orig "9router/proxy/internal/testnet"
	port "rust-anything.local/testnet_rs"
)

var mode = func() string {
	if os.Getenv("RUST_ANYTHING_UNIT_TESTNET") == "original" {
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

// Seal shadows the original `Seal`.
func Seal() {
	if mode == "new" {
		port.Seal()

	}
	orig.Seal()
	if mode != "old" {
		func() {
			defer func() {
				if r := recover(); r != nil {
					compare("Seal", []any{"no panic"}, []any{r})
				}
			}()
			port.Seal()
			compare("Seal", []any{}, []any{})
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
