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
