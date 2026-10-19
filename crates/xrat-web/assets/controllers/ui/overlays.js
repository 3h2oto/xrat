// @ts-check
import { closestControl, controls } from "../dom.js";
import { floating, closePopup, closeTooltip } from "./floating.js";

const overlaySelector =
  '[data-name="DialogContent"], [data-name="SheetContent"]';
const focusable =
  'button:not([disabled]):not([hidden]),input:not([disabled]),textarea:not([disabled]),select:not([disabled]),a[href],[tabindex="0"]';
/** @type {WeakMap<HTMLElement, Element | null>} */
const overlayOrigins = new WeakMap();
/** @type {Map<HTMLElement, boolean>} */
const savedInert = new Map();
/** @type {string | null} */
let savedOverflow = null;
/** @type {Element | null} */
let modalFocusReturn = null;
/** @returns {HTMLElement[]} */
export function activeOverlays() {
  return controls(document, overlaySelector).filter(
    (node) => node.dataset.state === "open",
  );
}
function syncModalIsolation() {
  savedInert.forEach((inert, node) => {
    if (node.isConnected) node.inert = inert;
  });
  savedInert.clear();
  const active = activeOverlays();
  if (!active.length) {
    if (savedOverflow !== null) document.body.style.overflow = savedOverflow;
    savedOverflow = null;
    return;
  }
  if (savedOverflow === null) savedOverflow = document.body.style.overflow;
  document.body.style.overflow = "hidden";
  active.forEach((panel, index) => {
    panel.style.zIndex = `${90 + index * 20}`;
    const backdrop = document.getElementById(`${panel.id}_backdrop`);
    if (backdrop) backdrop.style.zIndex = `${80 + index * 20}`;
  });
  const top = active.at(-1);
  if (!top) return;
  const backdrop = document.getElementById(`${top.id}_backdrop`);
  let branch = top;
  while (branch.parentElement) {
    for (const sibling of branch.parentElement.children) {
      if (
        !(sibling instanceof HTMLElement) ||
        sibling === branch ||
        sibling === backdrop ||
        sibling.contains(backdrop) ||
        sibling.tagName === "SCRIPT" ||
        sibling.tagName === "STYLE"
      )
        continue;
      savedInert.set(sibling, sibling.inert);
      sibling.inert = true;
    }
    branch = branch.parentElement;
  }
}
/**
 * @param {HTMLElement} panel
 * @param {Element | null} [trigger=document.activeElement]
 */
function openOverlay(panel, trigger = document.activeElement) {
  if (panel.dataset.state === "open") return;
  closePopup();
  closeTooltip();
  overlayOrigins.set(panel, trigger);
  modalFocusReturn = trigger;
  panel.inert = false;
  panel.dataset.state = "open";
  panel.style.pointerEvents = "auto";
  const backdrop = document.getElementById(`${panel.id}_backdrop`);
  if (backdrop) {
    backdrop.dataset.state = "open";
    backdrop.style.pointerEvents = "auto";
  }
  syncModalIsolation();
  (controls(panel, focusable)[0] || panel).focus();
}
/**
 * @param {HTMLElement} panel
 */
function closeOverlay(panel) {
  closePopup();
  closeTooltip();
  panel.dataset.state = "closed";
  panel.style.pointerEvents = "none";
  panel.inert = true;
  const backdrop = document.getElementById(`${panel.id}_backdrop`);
  if (backdrop) {
    backdrop.dataset.state = "closed";
    backdrop.style.pointerEvents = "none";
  }
  const origin = overlayOrigins.get(panel);
  controls(panel, "[data-overlay-notify-close]")[0]?.click();
  syncModalIsolation();
  if (origin instanceof HTMLElement && origin.isConnected) origin.focus();
  modalFocusReturn = null;
}
document.addEventListener("click", (event) => {
  const trigger = closestControl(
    event.target,
    "[data-dialog-trigger], [data-sheet-trigger]",
  );
  if (trigger) {
    const panel = document.getElementById(
      trigger.getAttribute("data-dialog-trigger") ||
        trigger.getAttribute("data-sheet-trigger") ||
        "",
    );
    if (panel) openOverlay(panel, trigger);
  }
  const closer = closestControl(
    event.target,
    "[data-dialog-close], [data-sheet-close]",
  );
  if (closer) {
    const panel = closestControl(closer, overlaySelector);
    if (panel) closeOverlay(panel);
  }
  const backdrop = closestControl(
    event.target,
    '[data-name="DialogBackdrop"], [data-name="SheetBackdrop"]',
  );
  if (backdrop) {
    const panel = document.getElementById(
      backdrop.id.replace(/_backdrop$/, ""),
    );
    if (panel?.dataset.backdrop === "auto") closeOverlay(panel);
  }
  if (
    closestControl(
      event.target,
      '[data-sheet-navigation="true"] [data-name="SidenavMenuButton"]',
    )
  ) {
    const panel = closestControl(event.target, '[data-name="SheetContent"]');
    if (panel) closeOverlay(panel);
  }
});
document.addEventListener("keydown", (event) => {
  const panel = activeOverlays().at(-1);
  if (!panel || floating.popup) return;
  if (event.key === "Escape") {
    event.preventDefault();
    closeOverlay(panel);
  }
  if (event.key === "Tab") {
    const nodes = controls(panel, focusable).filter(
      (node) => !node.closest("[inert]") && node.getClientRects().length,
    );
    const first = nodes[0];
    const last = nodes.at(-1);
    if (!first) {
      event.preventDefault();
      panel.focus();
    } else if (
      event.shiftKey &&
      (document.activeElement === first || document.activeElement === panel)
    ) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
});

const modalObserver = new MutationObserver(() => {
  controls(document, overlaySelector).forEach((panel) => {
    if (panel.dataset.autoOpen === "true" && !panel.dataset.initialized) {
      panel.dataset.initialized = "true";
      openOverlay(panel);
    }
  });
  if (savedOverflow !== null && !activeOverlays().length) {
    syncModalIsolation();
    if (modalFocusReturn instanceof HTMLElement && modalFocusReturn.isConnected)
      modalFocusReturn.focus();
    modalFocusReturn = null;
  }
});
modalObserver.observe(document.documentElement, {
  childList: true,
  subtree: true,
});
