use cosense_core::convert;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn markup_input_never_panics(
        s in r"[\[\]<>()/*#$~`\-_.:a-zA-Z0-9 \t\n\r　\u{a0}]{0,120}"
    ) {
        let _ = convert(&s);
    }

    #[test]
    fn arbitrary_input_never_panics(s in r"(?s).{0,200}") {
        let _ = convert(&s);
    }

    #[test]
    fn plain_words_are_preserved(s in r"[a-zA-Z0-9]([a-zA-Z0-9 ]{0,60}[a-zA-Z0-9])?") {
        let out = convert(&s);
        prop_assert_eq!(out, s);
    }

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

#[test]
fn deeply_nested_list_does_not_overflow() {
    let depth = 5000;
    let input: String = (1..=depth)
        .map(|n| format!("{}item", " ".repeat(n)))
        .collect::<Vec<_>>()
        .join("\n");

    let out = convert(&input);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), depth);
    assert_eq!(
        lines[depth - 1],
        format!("{}- item", "  ".repeat(depth - 1))
    );
}
