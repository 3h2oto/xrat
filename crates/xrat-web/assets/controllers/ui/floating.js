// @ts-check

/** @type {{ popup: HTMLElement | null, popupTrigger: HTMLElement | null, tooltip: HTMLElement | null }} */
export const floating = { popup: null, popupTrigger: null, tooltip: null };

// The native top layer escapes sticky/transform stacking contexts without moving Dioxus-owned nodes.
/**
 * @param {HTMLElement} content
 */
export function showFloating(content) {
  if (typeof content.showPopover !== "function") return;
  content.setAttribute("popover", "manual");
  if (!content.matches(":popover-open")) content.showPopover();
}
/**
 * @param {HTMLElement} content
 */
export function hideFloating(content) {
  if (
    typeof content.hidePopover === "function" &&
    content.matches(":popover-open")
  )
    content.hidePopover();
}
export function closeTooltip() {
  if (!floating.tooltip) return;
  hideFloating(floating.tooltip);
  floating.tooltip.style.opacity = "0";
  floating.tooltip.style.visibility = "hidden";
  floating.tooltip = null;
}

/**
 * @param {boolean} [restore=false]
 */
export function closePopup(restore = false) {
  if (!floating.popup) return;
  floating.popup.dataset.state = "closed";
  floating.popup.style.pointerEvents = "none";
  floating.popup.inert = true;
  hideFloating(floating.popup);
  floating.popupTrigger?.setAttribute("aria-expanded", "false");
  if (restore && floating.popupTrigger?.isConnected)
    floating.popupTrigger.focus();
  floating.popup = null;
  floating.popupTrigger = null;
}
