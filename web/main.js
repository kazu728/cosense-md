// Live preview: convert Cosense in the browser via the wasm core, then render.
//
// Output is pure Markdown — tags render as `#name`, links/images/autolinks as
// standard Markdown — so markdown-it runs with html: false and never injects raw
// HTML from the source. DOMPurify stays as defense-in-depth on the result.

import init, { convert } from "./pkg/cosense_wasm.js";

// markdown-it, DOMPurify, and KaTeX load from pinned CDN URLs with SRI as classic
// scripts in index.html, so they arrive as globals here.
const md = window.markdownit({ html: false, linkify: true });

const input = document.getElementById("input");
const preview = document.getElementById("preview");
const markdown = document.getElementById("markdown");
const copy = document.getElementById("copy");
const tabPreview = document.getElementById("tab-preview");
const tabMarkdown = document.getElementById("tab-markdown");

function render() {
  const src = convert(input.value);
  preview.innerHTML = window.DOMPurify.sanitize(md.render(src));
  // Typeset math into the DOM after sanitizing, so DOMPurify never sees (and so
  // cannot strip) KaTeX's output. `code:tex` blocks convert to `$$…$$`.
  window.renderMathInElement(preview, {
    delimiters: [{ left: "$$", right: "$$", display: true }],
    throwOnError: false,
  });
  markdown.value = src;
}

function selectTab(isMarkdown) {
  tabMarkdown.setAttribute("aria-selected", String(isMarkdown));
  tabPreview.setAttribute("aria-selected", String(!isMarkdown));
  markdown.hidden = !isMarkdown;
  preview.hidden = isMarkdown;
  copy.hidden = !isMarkdown;
}

await init();
input.addEventListener("input", render);
tabPreview.addEventListener("click", () => selectTab(false));
tabMarkdown.addEventListener("click", () => selectTab(true));
const copyLabel = copy.textContent;
let copyResetTimer;
copy.addEventListener("click", async () => {
  await navigator.clipboard.writeText(markdown.value);
  copy.textContent = "コピーしました";
  clearTimeout(copyResetTimer);
  copyResetTimer = setTimeout(() => { copy.textContent = copyLabel; }, 1200);
});
render();
