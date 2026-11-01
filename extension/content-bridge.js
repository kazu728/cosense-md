const TEXT_TYPE = "cosense-markdown-panel:text";
const SCROLL_TYPE = "cosense-markdown-panel:scroll";
const START_TYPE = "cosense-markdown-panel:start";
const STOP_TYPE = "cosense-markdown-panel:stop";

chrome.runtime.onConnect.addListener((port) => {
  const onMessage = (event) => {
    if (event.source !== window) return;
    const data = event.data;
    if (data?.type === TEXT_TYPE) {
      port.postMessage({ type: TEXT_TYPE, text: data.text });
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
