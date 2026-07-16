//! Golden conformance: the shared fixture JSON file is the single source of
//! truth. Every `{source, expected}` case must convert exactly.

use std::collections::BTreeMap;

use cosense_core::convert;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    id: String,
    source: Vec<String>,
    expected: Vec<String>,
}

#[test]
fn markdown_conformance() {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cosense_decision_table.json"
    );
    let raw = std::fs::read_to_string(fixture).expect("read fixture");
    let table: BTreeMap<String, Vec<Case>> =
        serde_json::from_str(&raw).expect("parse fixture json");

    let mut failures = Vec::new();
    for (category, cases) in &table {
        for case in cases {
            let source = case.source.join("\n");
            let expected = case.expected.join("\n");
            let got = convert(&source);
            if got != expected {
                failures.push(format!(
                    "[{}:{}]\n--- expected ---\n{}\n--- got ---\n{}",
                    category, case.id, expected, got
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} conformance case(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
