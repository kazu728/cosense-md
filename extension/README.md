# Cosense Markdown Panel

A Chrome extension (MV3) that shows a scrapbox.io page as Markdown in the browser **Side Panel**, live. Conversion runs in wasm (`crates/cosense-wasm`); the text comes from Cosense's UserScript API `scrapbox.Page.lines`, which Cosense keeps in sync over its own websocket — so the panel follows your edits (and other people's) in real time with no polling and no protocol re-implementation. The panel shows the raw Markdown text with a copy button; there is no rendered preview, so no markdown-it / DOMPurify / KaTeX is bundled.

## Build

```bash
# 0. First time only: the wasm-bindgen CLI (same version as the crate)
cargo install wasm-bindgen-cli --version 0.2.126

# 1. Package the wasm for the extension (generates extension/pkg/)
cargo build -p cosense-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/cosense_wasm.wasm \
  --out-dir extension/pkg --target web
```

`extension/pkg/` is generated, so it is not committed (already in `.gitignore`). Re-run step 1 after changing the core, then reload the extension.

## Load

`chrome://extensions` → enable Developer mode → **Load unpacked** → select `extension/`. Open any scrapbox.io page and click the toolbar icon to open the Side Panel.

## Known dependency

`scrapbox.Page.lines` is a semi-official API meant for UserScripts. If a Cosense change breaks it, the fallback is to poll the same-origin `GET /api/pages/:project/:title/text` endpoint instead (not implemented here).
