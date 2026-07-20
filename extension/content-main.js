// MAIN world: read the raw Cosense text from window.scrapbox.Page.lines, which
// Cosense keeps in sync over its own websocket, and relay it via postMessage.
// It also reports the scroll position (as a fraction over the total lines) so
// the panel can follow along. Both only run while a panel is watching: the
// bridge sends START on connect and STOP on disconnect.

const TEXT_TYPE = "cosense-markdown-panel:text";
const SCROLL_TYPE = "cosense-markdown-panel:scroll";
const START_TYPE = "cosense-markdown-panel:start";
const STOP_TYPE = "cosense-markdown-panel:stop";

function currentText() {
  const lines = window.scrapbox?.Page?.lines;
  return lines ? lines.map((line) => line.text).join("\n") : "";
}

function post() {
  window.postMessage({ type: TEXT_TYPE, text: currentText() }, location.origin);
}

// Scroll position measured in source lines: the index (plus the fraction of it
// scrolled past the top) of the first visible line, over the total line count.
// The source line count matches the Markdown line count closely enough that the
// panel can map this fraction onto its own scroll height. Line boxes stack top
// to bottom, so their bottom edge is monotonic and the first visible one can be
// found by binary search.
function scrollFraction() {
  const lines = document.querySelectorAll(".lines .line");
  const n = lines.length;
  if (n === 0) return 0;
  let lo = 0;
  let hi = n - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].getBoundingClientRect().bottom > 0) hi = mid;
    else lo = mid + 1;
  }
  const rect = lines[lo].getBoundingClientRect();
  const frac = rect.height > 0 ? Math.min(Math.max(-rect.top / rect.height, 0), 1) : 0;
  return (lo + frac) / n;
}

function postScroll() {
  window.postMessage({ type: SCROLL_TYPE, f: scrollFraction() }, location.origin);
}

let scrollPending = false;
function onScroll() {
  if (scrollPending) return;
  scrollPending = true;
  requestAnimationFrame(() => {
    scrollPending = false;
    postScroll();
  });
}

let observer;
let timer;
let watchers = 0; // connected panels; observe while > 0 so a stale STOP from an
                  // old connection can't halt updates a newer panel still wants

function start() {
  watchers += 1;
  if (!observer) {
    // scrapbox may not exist yet; the observer fires as the app renders, so no
    // explicit retry loop is needed.
    observer = new MutationObserver(() => {
      clearTimeout(timer);
      timer = setTimeout(post, 300);
    });
    observer.observe(document.body, { childList: true, characterData: true, subtree: true });
    window.addEventListener("scroll", onScroll, { passive: true, capture: true });
  }
  post(); // emit the current text and scroll position for the newly connected panel
  postScroll();
}

function stop() {
  watchers = Math.max(0, watchers - 1);
  if (watchers > 0) return;
  observer?.disconnect();
  observer = undefined;
  clearTimeout(timer);
  window.removeEventListener("scroll", onScroll, { capture: true });
}

window.addEventListener("message", (event) => {
  if (event.source !== window) return;
  if (event.data?.type === START_TYPE) start();
  else if (event.data?.type === STOP_TYPE) stop();
});
