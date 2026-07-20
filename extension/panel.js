// Side panel: while open, connect to the active tab's content script, convert
// the Cosense text via the wasm core, and show the Markdown. The connection is
// per-tab, so the content script only streams while this panel is watching it.

import init, { convert } from "./pkg/cosense_wasm.js";

const PORT = "cosense-markdown-panel";
const SCROLL_TYPE = "cosense-markdown-panel:scroll";
const HINT_CLOSED = "scrapbox.io のページを開いてください（開いている場合はタブを再読み込み）";
const HINT_EMPTY = "このページに表示できる本文がありません";
const ERROR_LOAD = "変換モジュールの読み込みに失敗しました。拡張機能を再読み込みしてください。";

const markdown = document.getElementById("markdown");
const placeholder = document.getElementById("placeholder");
const copy = document.getElementById("copy");

let lastText = null; // last Cosense text rendered; skip redundant re-render
let lastFraction = null; // last scroll position from the page, as a 0..1 fraction

function render(text) {
  if (text === lastText) return;
  lastText = text;
  const hasText = text.length > 0;
  if (hasText) markdown.value = convert(text);
  markdown.hidden = !hasText;
  copy.hidden = !hasText;
  placeholder.hidden = hasText;
  if (hasText) applyScroll(); // value replacement resets scrollTop; restore it
  else placeholder.textContent = HINT_EMPTY;
}

function applyScroll() {
  if (lastFraction === null || markdown.hidden) return;
  // lastFraction is a position within the content (0 = top line, 1 = last line),
  // so map it onto the full content height and let the browser clamp scrollTop.
  const f = Math.min(Math.max(lastFraction, 0), 1);
  markdown.scrollTop = f * markdown.scrollHeight;
}

function showPlaceholder(text) {
  lastText = null;
  markdown.hidden = true;
  copy.hidden = true;
  placeholder.hidden = false;
  placeholder.textContent = text;
}

let port = null;
let connectSeq = 0;

function connectToActiveTab() {
  port?.disconnect();
  port = null;
  lastFraction = null; // don't carry the previous tab's scroll into the new one
  const seq = ++connectSeq;

  chrome.tabs.query({ active: true, currentWindow: true }).then(([tab]) => {
    if (seq !== connectSeq) return; // a newer connect superseded this one
    if (!tab?.id) return showPlaceholder(HINT_CLOSED);

    const active = chrome.tabs.connect(tab.id, { name: PORT });
    let gotText = false;

    active.onMessage.addListener((message) => {
      if (message.type === SCROLL_TYPE) {
        lastFraction = message.f;
        applyScroll();
        return;
      }
      gotText = true;
      render(message.text);
    });
    active.onDisconnect.addListener(() => {
      void chrome.runtime.lastError; // expected when the tab has no content script
      if (active !== port) return;
      port = null;
      if (!gotText) showPlaceholder(HINT_CLOSED);
    });

    port = active;
  });
}

const copyLabel = copy.textContent;
let copyResetTimer;
copy.addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(markdown.value);
    copy.textContent = "コピーしました";
  } catch {
    copy.textContent = "コピーできませんでした";
  }
  clearTimeout(copyResetTimer);
  copyResetTimer = setTimeout(() => { copy.textContent = copyLabel; }, 1200);
});

async function main() {
  await init();
  const { id: windowId } = await chrome.windows.getCurrent();

  chrome.tabs.onActivated.addListener((activeInfo) => {
    if (activeInfo.windowId === windowId) connectToActiveTab();
  });
  chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    if (tab.active && tab.windowId === windowId && changeInfo.status === "complete") {
      connectToActiveTab();
    }
  });

  connectToActiveTab();
}

main().catch((error) => {
  showPlaceholder(ERROR_LOAD);
  console.error(error);
});
