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

function render() {
  preview.innerHTML = DOMPurify.sanitize(md.render(convert(input.value)));
}

await init();
input.addEventListener("input", render);
render();
