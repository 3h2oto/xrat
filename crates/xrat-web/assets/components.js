// @ts-check
// CSP-compatible behavior for the vendored Rust/UI component data attributes.
/**
 * Find a component control from a delegated event target.
 * @param {EventTarget | null} target
 * @param {string} selector
 * @returns {HTMLElement | null}
 */
function closestControl(target, selector) {
  if (!(target instanceof Element)) return null;
  const control = target.closest(selector);
  return control instanceof HTMLElement ? control : null;
}

/**
 * @param {ParentNode} root
 * @param {string} selector
 * @returns {HTMLElement[]}
 */
function controls(root, selector) {
  return [...root.querySelectorAll(selector)].filter(
    (node) => node instanceof HTMLElement,
  );
}

const popupNames = ["SelectContent", "DropdownMenuContent"];
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
const popupSelector = popupNames
  .map((name) => `[data-name="${name}"]`)
  .join(",");
/** @type {HTMLElement | null} */
let popup = null;
/** @type {HTMLElement | null} */
let popupTrigger = null;

// The native top layer escapes sticky/transform stacking contexts without moving Dioxus-owned nodes.
/**
 * @param {HTMLElement} content
 */
function showFloating(content) {
  if (typeof content.showPopover !== "function") return;
  content.setAttribute("popover", "manual");
  if (!content.matches(":popover-open")) content.showPopover();
}
/**
 * @param {HTMLElement} content
 */
function hideFloating(content) {
  if (
    typeof content.hidePopover === "function" &&
    content.matches(":popover-open")
  )
    content.hidePopover();
}
/** @type {HTMLElement | null} */
let tooltip = null;
function closeTooltip() {
  if (!tooltip) return;
  hideFloating(tooltip);
  tooltip.style.opacity = "0";
  tooltip.style.visibility = "hidden";
  tooltip = null;
}

/**
 * @param {HTMLElement} content
 * @returns {HTMLElement[]}
 */
function options(content) {
  return controls(
    content,
    '[data-select-option], [data-dropdown-close], [role="menuitem"], [role="menuitemcheckbox"]',
  ).filter((node) => !node.matches(":disabled"));
}
/**
 * @param {boolean} [restore=false]
 */
function closePopup(restore = false) {
  if (!popup) return;
  popup.dataset.state = "closed";
  popup.style.pointerEvents = "none";
  popup.inert = true;
  hideFloating(popup);
  popupTrigger?.setAttribute("aria-expanded", "false");
  if (restore && popupTrigger?.isConnected) popupTrigger.focus();
  popup = null;
  popupTrigger = null;
}
/**
 * @param {HTMLElement} content
 * @param {HTMLElement | null | undefined} trigger
 * @param {boolean} [focus=false]
 */
function openPopup(content, trigger, focus = false) {
  if (!trigger || popup === content) return;
  closePopup();
  closeTooltip();
  popup = content;
  popupTrigger = trigger;
  content.inert = false;
  content.dataset.state = "open";
  content.style.pointerEvents = "auto";
  trigger.setAttribute("aria-expanded", "true");
  const rect = trigger.getBoundingClientRect();
  const preferredWidth =
    content.dataset.name === "SelectContent"
      ? rect.width
      : Math.max(rect.width, 180);
  const width = Math.min(preferredWidth, innerWidth - 16);
  content.style.width = `${width}px`;
  content.style.minWidth = "0";
  content.style.margin = "0";
  showFloating(content);
  const desired = Math.min(content.scrollHeight, 300);
  const above =
    innerHeight - rect.bottom < desired && rect.top > innerHeight - rect.bottom;
  content.style.position = "fixed";
  content.style.width = `${width}px`;
  content.style.minWidth = "0";
  content.style.left = `${Math.max(8, Math.min(rect.left, innerWidth - width - 8))}px`;
  content.style.right = "auto";
  content.style.top = above ? "auto" : `${rect.bottom + 4}px`;
  content.style.bottom = above ? `${innerHeight - rect.top + 4}px` : "auto";
  content.style.maxHeight = `${Math.max(60, Math.min(300, (above ? rect.top : innerHeight - rect.bottom) - 12))}px`;
  content.style.overflowY = "auto";
  content.style.overflowX = "hidden";
  if (content.dataset.align === "start-outer") {
    const left =
      rect.right + 4 + width <= innerWidth - 8
        ? rect.right + 4
        : rect.left - width - 4;
    content.style.left = `${Math.max(8, left)}px`;
    content.style.top = `${Math.max(8, Math.min(rect.top, innerHeight - desired - 8))}px`;
    content.style.bottom = "auto";
    content.style.maxHeight = `${innerHeight - 16}px`;
  }
  if (focus)
    (
      controls(content, '[aria-selected="true"]')[0] || options(content)[0]
    )?.focus();
}
/**
 * @param {HTMLElement} content
 * @returns {HTMLElement | null}
 */
function triggerFor(content) {
  const attribute =
    content.dataset.name === "SelectContent"
      ? "data-select-trigger"
      : "data-dropdown-trigger";
  const wrapper = document.querySelector(
    `[${attribute}="${CSS.escape(content.id)}"]`,
  );
  return wrapper instanceof HTMLElement && wrapper.matches("button,a")
    ? wrapper
    : wrapper
      ? (controls(wrapper, "button,a")[0] ?? null)
      : null;
}

document.addEventListener("click", (event) => {
  const trigger = closestControl(
    event.target,
    "[data-select-trigger], [data-dropdown-trigger]",
  );
  if (trigger) {
    const id =
      trigger.getAttribute("data-select-trigger") ||
      trigger.getAttribute("data-dropdown-trigger");
    const content = document.getElementById(id || "");
    if (content && !trigger.matches(":disabled")) {
      if (popup === content) {
        if (trigger.closest('[data-hover-menu="true"]'))
          options(content)[0]?.focus();
        else closePopup();
      } else
        openPopup(
          content,
          trigger.matches("button,a")
            ? trigger
            : controls(trigger, "button,a")[0],
          true,
        );
    }
    return;
  }
  if (
    closestControl(event.target, "[data-select-option], [data-dropdown-close]")
  )
    closePopup(true);
  else if (
    popup &&
    !popup.contains(event.target instanceof Node ? event.target : null)
  )
    closePopup();
});

document.addEventListener("keydown", (event) => {
  const sample =
    event.target instanceof Element
      ? event.target.closest(".chart-sample")
      : null;
  if (
    sample &&
    ["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)
  ) {
    const samples = [
      ...(sample.closest("svg")?.querySelectorAll("g") ?? []),
    ].filter((node) => node.classList.contains("chart-sample"));
    const index = samples.findIndex((node) => node === sample);
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? samples.length - 1
          : Math.max(
              0,
              Math.min(
                samples.length - 1,
                index + (event.key === "ArrowRight" ? 1 : -1),
              ),
            );
    event.preventDefault();
    samples[next]?.focus();
    return;
  }
  const trigger = closestControl(
    event.target,
    "[data-select-trigger], [data-dropdown-trigger]",
  );
  if (!popup && trigger && ["ArrowDown", "ArrowUp"].includes(event.key)) {
    const content = document.getElementById(
      trigger.getAttribute("data-select-trigger") ||
        trigger.getAttribute("data-dropdown-trigger") ||
        "",
    );
    if (content && !trigger.matches(":disabled")) {
      event.preventDefault();
      openPopup(
        content,
        trigger.matches("button,a")
          ? trigger
          : controls(trigger, "button,a")[0],
        true,
      );
    }
    return;
  }
  if (!popup) {
    if (event.key === "Escape" && tooltip) {
      closeTooltip();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
    return;
  }
  const items = options(popup);
  const index = items.findIndex((node) => node === document.activeElement);
  if (event.key === "Escape") {
    event.preventDefault();
    event.stopImmediatePropagation();
    closePopup(true);
  } else if (event.key === "Tab") closePopup(true);
  else if (
    ["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key) &&
    items.length
  ) {
    event.preventDefault();
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? items.length - 1
          : (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) %
            items.length;
    items[next].focus();
    items[next].scrollIntoView({ block: "nearest" });
  } else if (
    ["Enter", " "].includes(event.key) &&
    document.activeElement instanceof HTMLElement &&
    items.includes(document.activeElement)
  ) {
    event.preventDefault();
    document.activeElement.click();
  } else if (
    event.key.length === 1 &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.altKey &&
    items.length
  ) {
    const next = [...items.slice(index + 1), ...items.slice(0, index + 1)].find(
      (node) =>
        (node.textContent ?? "")
          .trim()
          .toLowerCase()
          .startsWith(event.key.toLowerCase()),
    );
    if (next) {
      event.preventDefault();
      next.focus();
      next.scrollIntoView({ block: "nearest" });
    }
  }
});
document.addEventListener("focusin", (event) => {
  if (
    popup &&
    !popup.contains(event.target instanceof Node ? event.target : null) &&
    !popupTrigger?.contains(event.target instanceof Node ? event.target : null)
  )
    closePopup();
});
window.addEventListener("resize", () => {
  closePopup();
  closeTooltip();
});
document.addEventListener(
  "scroll",
  (event) => {
    closeTooltip();
    if (
      popup &&
      event.target !== popup &&
      !popup.contains(event.target instanceof Node ? event.target : null)
    )
      closePopup();
  },
  true,
);

// Initialization is delegated, so remounting forms does not leak listeners.
const observer = new MutationObserver(() => {
  if (popup && !popup.isConnected) closePopup();
  if (tooltip && !tooltip.isConnected) closeTooltip();
  controls(document, popupSelector).forEach((content) => {
    if (content.dataset.state !== "open") content.inert = true;
    const trigger = triggerFor(content);
    if (trigger) {
      trigger.setAttribute("aria-controls", content.id);
      trigger.setAttribute(
        "aria-expanded",
        content.dataset.state === "open" ? "true" : "false",
      );
    }
  });
});
observer.observe(document.documentElement, { childList: true, subtree: true });

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
function activeOverlays() {
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
  if (!panel || popup) return;
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

/**
 * @param {HTMLElement} root
 */
function positionTooltip(root) {
  const content = controls(root, '[role="tooltip"]')[0];
  if (!content || popup) return;
  if (tooltip && tooltip !== content) closeTooltip();
  tooltip = content;
  content.style.opacity = "1";
  content.style.visibility = "visible";
  showFloating(content);
  const rect = root.getBoundingClientRect();
  const size = content.getBoundingClientRect();
  const side = content.dataset.position;
  let left =
    side === "Left"
      ? rect.left - size.width - 10
      : side === "Right"
        ? rect.right + 10
        : rect.left + (rect.width - size.width) / 2;
  let top =
    side === "Top"
      ? rect.top - size.height - 8
      : side === "Bottom"
        ? rect.bottom + 8
        : rect.top + (rect.height - size.height) / 2;
  content.style.position = "fixed";
  content.style.left = `${Math.max(8, Math.min(left, innerWidth - size.width - 8))}px`;
  content.style.top = `${Math.max(8, Math.min(top, innerHeight - size.height - 8))}px`;
  content.style.bottom = "auto";
  content.style.right = "auto";
  content.style.margin = "0";
}
document.addEventListener("mouseover", (event) => {
  const tooltip = closestControl(event.target, '[data-name="Tooltip"]');
  if (tooltip) positionTooltip(tooltip);
  const menu = closestControl(event.target, '[data-hover-menu="true"]');
  if (!menu) return;
  const content = controls(menu, '[data-name="DropdownMenuContent"]')[0];
  if (content) openPopup(content, triggerFor(content), false);
});
document.addEventListener("mouseout", (event) => {
  const tipRoot = tooltip?.closest('[data-name="Tooltip"]');
  if (
    tipRoot &&
    !tipRoot.contains(
      event.relatedTarget instanceof Node ? event.relatedTarget : null,
    ) &&
    !tipRoot.contains(document.activeElement)
  )
    closeTooltip();
  if (popup?.dataset.name !== "DropdownMenuContent") return;
  const menu = popup.closest('[data-hover-menu="true"]');
  if (
    menu &&
    !menu.contains(
      event.relatedTarget instanceof Node ? event.relatedTarget : null,
    ) &&
    !popup.contains(document.activeElement)
  ) {
    const leaving = popup;
    setTimeout(() => {
      if (
        popup === leaving &&
        !menu.matches(":hover") &&
        !leaving.contains(document.activeElement)
      )
        closePopup();
    }, 120);
  }
});
document.addEventListener("focusin", (event) => {
  const tipRoot = tooltip?.closest('[data-name="Tooltip"]');
  if (
    tipRoot &&
    !tipRoot.contains(event.target instanceof Node ? event.target : null)
  )
    closeTooltip();
  const tooltipRoot = closestControl(event.target, '[data-name="Tooltip"]');
  if (tooltipRoot) positionTooltip(tooltipRoot);
  const menu = closestControl(event.target, '[data-hover-menu="true"]');
  if (menu) {
    const content = controls(menu, '[data-name="DropdownMenuContent"]')[0];
    if (content) openPopup(content, triggerFor(content), false);
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

document.addEventListener("click", (event) => {
  const button = closestControl(event.target, "[data-log-top]");
  if (button)
    document
      .getElementById(button.dataset.logTop ?? "")
      ?.scrollTo({ top: 0, left: 0, behavior: "auto" });
});
