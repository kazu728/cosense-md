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
    let table: BTreeMap<String, Vec<Case>> =
        serde_json::from_str(include_str!("conformance.json")).expect("parse fixture json");

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
