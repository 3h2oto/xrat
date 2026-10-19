// @ts-check
import { closestControl } from "../dom.js";
import { activeOverlays } from "../ui/overlays.js";

document.addEventListener("paste", (event) => {
  if (
    event.defaultPrevented ||
    !document.querySelector('[data-import-paste="true"]')
  )
    return;
  if (
    closestControl(
      event.target,
      'input,textarea,select,[contenteditable]:not([contenteditable="false"]),[role="textbox"]',
    ) ||
    activeOverlays().length
  )
    return;
  const text = event.clipboardData?.getData("text/plain");
  if (!text?.trim()) return;
  event.preventDefault();
  window.dispatchEvent(new CustomEvent("xrat-import-paste", { detail: text }));
});
