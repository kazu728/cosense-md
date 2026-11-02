const TEXT_TYPE = "cosense-markdown-panel:text";
const SCROLL_TYPE = "cosense-markdown-panel:scroll";
const START_TYPE = "cosense-markdown-panel:start";
const STOP_TYPE = "cosense-markdown-panel:stop";
const PORT = "cosense-markdown-panel";

chrome.runtime.onConnect.addListener((port) => {
  if (port.name !== PORT) return;

  const onMessage = (event) => {
    if (event.source !== window) return;
    const type = event.data?.type;
    if (type === TEXT_TYPE || type === SCROLL_TYPE) port.postMessage(event.data);
  };

  window.addEventListener("message", onMessage);
  window.postMessage({ type: START_TYPE }, location.origin);

  port.onDisconnect.addListener(() => {
    window.removeEventListener("message", onMessage);
    window.postMessage({ type: STOP_TYPE }, location.origin);
  });
});
