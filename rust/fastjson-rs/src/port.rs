//! The Rust port of `internal/fastjson/`: deterministic JSON marshaling shared by the proxy
//! hot paths (chat translation, upstream payload repair, semantic-cache key building).
//!
//! Two non-negotiable properties (from the Go source):
//!   - Deterministic output: map members are sorted by name, so serializing the same value
//!     twice yields identical bytes.
//!   - Identical error surface: type mismatches on unmarshal propagate as errors.
//!
//! `serde_json` without `preserve_order` serializes `Map` as a `BTreeMap` (sorted keys) and
//! does not HTML-escape `<`/`>`/`&`, so it matches fastjson's contract exactly.

#![forbid(unsafe_code)]

use serde::{Serialize, de::DeserializeOwned};
use std::io::Read;

// Marshal serializes `value` to JSON, sorting map members by name.
pub fn marshal<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| e.to_string())
}

// Unmarshal deserializes `data` into a value of type `T`, mirroring `Unmarshal(data []byte, v any)`.
pub fn unmarshal<T: DeserializeOwned>(data: &[u8]) -> Result<T, String> {
    serde_json::from_slice(data).map_err(|e| e.to_string())
}

// UnmarshalRead deserializes a single JSON value read from `read` into a value of type `T`,
// mirroring `UnmarshalRead(r io.Reader, v any)`.
pub fn unmarshal_read<T: DeserializeOwned>(read: &mut dyn Read) -> Result<T, String> {
    let mut data = Vec::new();
    read.read_to_end(&mut data).map_err(|e| e.to_string())?;
    serde_json::from_slice(&data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Sample {
        name: String,
        count: i64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,
    }

    #[test]
    fn marshal_unmarshal_round_trip() {
        let in_ = Sample {
            name: "9router".to_string(),
            count: 42,
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let data = marshal(&in_).expect("marshal");
        let out: Sample = unmarshal(&data).expect("unmarshal");
        assert_eq!(out, in_);
    }

    #[test]
    fn marshal_no_html_escape() {
        let data = marshal(&serde_json::json!({"expr": "1 < 2 && 3 > 2"})).expect("marshal");
        assert!(
            data.contains(&b'<'),
            "expected literal '<' (not HTML-escaped), got {:?}",
            data
        );
    }

    #[test]
    fn marshal_is_deterministic() {
        let v: serde_json::Value = serde_json::json!({"z": 1, "a": 2, "m": 3, "b": 4});
        let first = marshal(&v).expect("marshal");
        assert_eq!(first, b"{\"a\":2,\"b\":4,\"m\":3,\"z\":1}");
        for _ in 0..50 {
            let again = marshal(&v).expect("marshal");
            assert_eq!(again, first, "marshal not deterministic");
        }
    }

    #[test]
    fn unmarshal_type_mismatch_errors() {
        let err: Result<Sample, String> = unmarshal(r#"{"count": "not-an-int"}"#.as_bytes());
        assert!(err.is_err(), "expected error unmarshaling wrong type");
    }

    #[test]
    fn unmarshal_read_from_reader() {
        let mut reader = std::io::Cursor::new(r#"{"name":"read_test","count":10}"#.as_bytes());
        let out: Sample = unmarshal_read(&mut reader).expect("unmarshal_read");
        assert_eq!(out.name, "read_test");
        assert_eq!(out.count, 10);
    }
}
