# cosense-md

Convert [Cosense](https://scrapbox.io/) notation to Markdown. A single Rust conversion core drives every frontend, so all frontends stay in lockstep.

## Layout

| Path | What |
|---|---|
| [`crates/cosense-core`](crates/cosense-core) | The conversion engine (Cosense → Markdown). Pure Rust, no bindings. |
| [`crates/cosense-wasm`](crates/cosense-wasm) | wasm-bindgen shim over the core, used by the web preview and the Chrome extension. |
| [`markitdown-cosense`](markitdown-cosense) | The PyPI package: a [MarkItDown](https://github.com/microsoft/markitdown) plugin wrapping the core via PyO3. |
| [`web`](web) | Browser live preview built on the wasm module. |
| [`extension`](extension) | Chrome side panel that shows the open scrapbox.io page as Markdown, built on the wasm module. |

## Development

- Toolchain: `nix develop` provides everything below, including the matching `wasm-bindgen-cli`.
- Rust core: `cargo test -p cosense-core` (golden `{source, expected}` cases in `tests/conformance.json` + property tests).
- Python plugin: `uv run pytest` in `markitdown-cosense/` (builds the PyO3 module and rebuilds it when Rust sources change).
- Web preview: see [`web/README.md`](web/README.md).
- Chrome extension: see [`extension/README.md`](extension/README.md).

## License

MIT © kazu728
