import init, { convert } from "./pkg/cosense_wasm.js";
import { mathBlocks } from "./math.js";

const md = window.markdownit({ linkify: true }).use(mathBlocks);

const input = document.getElementById("input");
const preview = document.getElementById("preview");
const markdown = document.getElementById("markdown");
const copy = document.getElementById("copy");
const tabPreview = document.getElementById("tab-preview");
const tabMarkdown = document.getElementById("tab-markdown");

function render() {
  const src = convert(input.value);
  preview.innerHTML = window.DOMPurify.sanitize(md.render(src));
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
  try {
    await navigator.clipboard.writeText(markdown.value);
    copy.textContent = "Copied";
  } catch {
    copy.textContent = "Copy failed";
  }
  clearTimeout(copyResetTimer);
  copyResetTimer = setTimeout(() => { copy.textContent = copyLabel; }, 1200);
});
render();
