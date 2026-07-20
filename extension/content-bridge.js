// ISOLATED world: bridge between the page (postMessage) and the panel (Port).
// A panel connects a Port only while it is open; the bridge relays text and
// scroll updates from the MAIN-world script to that Port, and tells the
// MAIN-world script to start/stop working so nothing runs when no panel is
// watching.

const TEXT_TYPE = "cosense-markdown-panel:text";
const SCROLL_TYPE = "cosense-markdown-panel:scroll";
const START_TYPE = "cosense-markdown-panel:start";
const STOP_TYPE = "cosense-markdown-panel:stop";
const PORT = "cosense-markdown-panel";

chrome.runtime.onConnect.addListener((port) => {
  if (port.name !== PORT) return;

  const onMessage = (event) => {
    if (event.source !== window || event.origin !== location.origin) return;
    const data = event.data;
    if (data?.type === TEXT_TYPE) {
      port.postMessage({ type: TEXT_TYPE, text: String(data.text ?? "") });
    } else if (data?.type === SCROLL_TYPE) {
      port.postMessage({ type: SCROLL_TYPE, f: data.f });
    }
  };

  window.addEventListener("message", onMessage);
  window.postMessage({ type: START_TYPE }, location.origin);

  port.onDisconnect.addListener(() => {
    window.removeEventListener("message", onMessage);
    window.postMessage({ type: STOP_TYPE }, location.origin);
  });
});
