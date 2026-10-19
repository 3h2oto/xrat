// @ts-check
import { closestControl } from "../dom.js";

document.addEventListener("click", async (event) => {
  const button = closestControl(event.target, "[data-copy-textarea]");
  if (!button) return;
  const textarea = document.getElementById(button.dataset.copyTextarea ?? "");
  if (!textarea) return;
  const label = button.querySelector("span") || button;
  const originalLabel = label.textContent;
  const tooltip = closestControl(
    button,
    '[data-name="Tooltip"]',
  )?.querySelector('[role="tooltip"]');
  const originalTooltip = tooltip?.textContent;
  try {
    if (navigator.clipboard && window.isSecureContext)
      await navigator.clipboard.writeText(
        textarea instanceof HTMLTextAreaElement
          ? textarea.value
          : (textarea.textContent ?? ""),
      );
    else {
      textarea.focus();
      const selection = window.getSelection();
      if (textarea instanceof HTMLTextAreaElement) {
        textarea.select();
      } else {
        const range = document.createRange();
        range.selectNodeContents(textarea);
        selection?.removeAllRanges();
        selection?.addRange(range);
      }
      if (!document.execCommand("copy")) throw new Error("Copy failed");
      selection?.removeAllRanges();
      button.focus({ preventScroll: true });
    }
    label.textContent = "Copied";
  } catch {
    label.textContent = "Copy manually";
  }
  if (tooltip) tooltip.textContent = label.textContent;
  setTimeout(() => {
    if (!button.isConnected) return;
    label.textContent = originalLabel;
    if (tooltip) tooltip.textContent = originalTooltip ?? originalLabel;
  }, 2500);
});
