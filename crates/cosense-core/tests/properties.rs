//! Property-based invariants that golden cases cannot express: the converter is
//! total (never panics) and does not silently drop body text.

use cosense_core::convert;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Any string built from the cosense markup alphabet converts without panic.
    #[test]
    fn markup_input_never_panics(
        s in r"[\[\]<>()/*#$~`\-_.:a-zA-Z0-9 \t\n　]{0,120}"
    ) {
        let _ = convert(&s);
    }

    /// Arbitrary Unicode (including newlines) converts without panic.
    #[test]
    fn arbitrary_input_never_panics(s in r"(?s).{0,200}") {
        let _ = convert(&s);
    }

    /// A single line of plain word characters carries through unchanged: no
    /// markup means nothing to transform and nothing to drop.
    #[test]
    fn plain_words_are_preserved(s in r"[a-zA-Z0-9]([a-zA-Z0-9 ]{0,60}[a-zA-Z0-9])?") {
        let out = convert(&s);
        prop_assert_eq!(out, s);
    }

    /// Several marker-free lines joined into one paragraph are preserved
    /// verbatim. The single-line case above cannot see reflow bugs — a bracket
    /// fusing two lines, a line splitting — that only appear across a newline.
    #[test]
    fn plain_multiline_is_preserved(
        lines in prop::collection::vec(
            r"[a-zA-Z0-9]([a-zA-Z0-9 ]{0,40}[a-zA-Z0-9])?",
            1..6,
        )
    ) {
        let input = lines.join("\n");
        let out = convert(&input);
        prop_assert_eq!(out, input);
    }
}

/// A list nested thousands deep converts instead of overflowing the stack. List
/// nesting is handled with an explicit stack and a flat item vector — no
/// recursion over depth in the build, render, or drop — so depth costs no call
/// frames. Guards the totality claim in `lib.rs`.
#[test]
fn deeply_nested_list_does_not_overflow() {
    let depth = 5000;
    // Each line is indented one space deeper than the last, so it nests one level
    // further: a strictly increasing chain that a recursive parser/renderer would
    // descend to the bottom of.
    let input: String = (1..=depth)
        .map(|n| format!("{}item", " ".repeat(n)))
        .collect::<Vec<_>>()
        .join("\n");

    let out = convert(&input);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), depth);
    // Levels run 0..depth, so the deepest bullet has depth-1 indent units.
    assert_eq!(
        lines[depth - 1],
        format!("{}- item", "  ".repeat(depth - 1))
    );
}
