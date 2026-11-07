# Cosense live preview

A static page that converts Cosense to Markdown in the browser and previews it instantly. Conversion runs in wasm (`crates/cosense-wasm`); markdown-it renders the Markdown, DOMPurify sanitizes it, and KaTeX typesets math (`code:tex` → `$$…$$`). A block rule preserves math source through Markdown rendering, including blank lines and embedded dollar signs. Those three libraries load from pinned jsdelivr URLs with **SRI (`integrity="sha384-…"`)**, so the browser rejects any tampered bytes on a hash mismatch.

## Build & serve

```bash
# 0. Install once. Match the crate version.
cargo install wasm-bindgen-cli --version 0.2.130

# 1. Build the wasm package.
cargo build -p cosense-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/cosense_wasm.wasm \
  --out-dir web/pkg --target web

# 2. Serve over HTTP. file:// is unsupported.
python3 -m http.server -d web 8000
# -> http://localhost:8000
```

`web/pkg/` is generated, so it is not committed (already in `.gitignore`). Re-run step 1 after changing the core.

Run the math parser checks with `node --test web/math.test.mjs` from the repository root. They load the same pinned markdown-it script as the page and verify its SRI hash; network access is required.

## Third-party libraries

markdown-it, DOMPurify, and KaTeX load from **pinned jsdelivr URLs** with **SRI (`integrity`)** on each `<script>` / `<link>`. They are self-contained bundles with no transitive imports, so SRI covers them whole; the only unpinned surface is the woff2 fonts KaTeX pulls in via CSS, which are non-executable. When bumping a version, update the `integrity` sha384 alongside the URL:

```bash
curl -s <url> | openssl dgst -sha384 -binary | openssl base64 -A
```
