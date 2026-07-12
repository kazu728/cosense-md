// Live preview: convert Cosense in the browser via the wasm core, then render.
//
// Output is pure Markdown — tags render as `#name`, links/images/autolinks as
// standard Markdown — so markdown-it runs with html: false and never injects raw
// HTML from the source. DOMPurify stays as defense-in-depth on the result.

import init, { convert } from "./pkg/cosense_wasm.js";
import MarkdownIt from "https://esm.sh/markdown-it@14";
import DOMPurify from "https://esm.sh/dompurify@3";

const md = new MarkdownIt({ html: false, linkify: true });

const input = document.getElementById("input");
const preview = document.getElementById("preview");
const markdown = document.getElementById("markdown");
const copy = document.getElementById("copy");
const tabPreview = document.getElementById("tab-preview");
const tabMarkdown = document.getElementById("tab-markdown");

function render() {
  const src = convert(input.value);
  preview.innerHTML = DOMPurify.sanitize(md.render(src));
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
copy.addEventListener("click", async () => {
  await navigator.clipboard.writeText(markdown.value);
  const prev = copy.textContent;
  copy.textContent = "コピーしました";
  setTimeout(() => { copy.textContent = prev; }, 1200);
});
render();
