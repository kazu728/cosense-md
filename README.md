# cosense-md

Convert [Cosense](https://scrapbox.io/) notation to Markdown. A single Rust conversion core drives every frontend, so all frontends stay in lockstep.

## Layout

| Path | What |
|---|---|
| [`crates/cosense-core`](crates/cosense-core) | The conversion engine (Cosense → Markdown). Pure Rust, no bindings. |
| [`crates/cosense-wasm`](crates/cosense-wasm) | wasm-bindgen shim over the core, used by the web preview. |
| [`markitdown-cosense`](markitdown-cosense) | The PyPI package: a [MarkItDown](https://github.com/microsoft/markitdown) plugin wrapping the core via PyO3. |
| [`web`](web) | Browser live preview built on the wasm module. |
| [`fixtures`](fixtures) | Language-neutral `{source, expected}` golden tables — the single source of truth exercised by both the Rust conformance suite and the Python tests. |

## Development

- Rust core: `cargo test -p cosense-core` (golden conformance + property tests).
- Python plugin: see [`markitdown-cosense/README.md`](markitdown-cosense/README.md).
- Web preview: see [`web/README.md`](web/README.md).

## License

MIT © kazu728
