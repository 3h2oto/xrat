// @ts-check
import { closestControl, controls } from "../dom.js";
import {
  floating,
  closePopup,
  closeTooltip,
  showFloating,
} from "./floating.js";

const popupNames = ["SelectContent", "DropdownMenuContent"];
const popupSelector = popupNames
  .map((name) => `[data-name="${name}"]`)
  .join(",");
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
 * @param {HTMLElement} content
 * @param {HTMLElement | null | undefined} trigger
 * @param {boolean} [focus=false]
 */
function openPopup(content, trigger, focus = false) {
  if (!trigger || floating.popup === content) return;
  closePopup();
  closeTooltip();
  floating.popup = content;
  floating.popupTrigger = trigger;
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
      if (floating.popup === content) {
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
    floating.popup &&
    !floating.popup.contains(event.target instanceof Node ? event.target : null)
  )
    closePopup();
});

document.addEventListener("keydown", (event) => {
  if (
    event.target instanceof Element &&
    event.target.closest(".chart-sample") &&
    ["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)
  )
    return;
  const trigger = closestControl(
    event.target,
    "[data-select-trigger], [data-dropdown-trigger]",
  );
  if (
    !floating.popup &&
    trigger &&
    ["ArrowDown", "ArrowUp"].includes(event.key)
  ) {
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
  if (!floating.popup) {
    if (event.key === "Escape" && floating.tooltip) {
      closeTooltip();
      event.preventDefault();
      event.stopImmediatePropagation();
    }
    return;
  }
  if (
    event.target instanceof HTMLElement &&
    (event.target.matches("input,textarea") ||
      event.target.isContentEditable) &&
    !["Escape", "Tab", "ArrowDown"].includes(event.key)
  )
    return;
  const items = options(floating.popup);
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
    floating.popup &&
    !floating.popup.contains(
      event.target instanceof Node ? event.target : null,
    ) &&
    !floating.popupTrigger?.contains(
      event.target instanceof Node ? event.target : null,
    )
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
      floating.popup &&
      event.target !== floating.popup &&
      !floating.popup.contains(
        event.target instanceof Node ? event.target : null,
      )
    )
      closePopup();
  },
  true,
);

// Initialization is delegated, so remounting forms does not leak listeners.
const observer = new MutationObserver(() => {
  if (floating.popup && !floating.popup.isConnected) closePopup();
  if (floating.tooltip && !floating.tooltip.isConnected) closeTooltip();
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

document.addEventListener("mouseover", (event) => {
  const menu = closestControl(event.target, '[data-hover-menu="true"]');
  if (!menu) return;
  const content = controls(menu, '[data-name="DropdownMenuContent"]')[0];
  if (content) openPopup(content, triggerFor(content), false);
});
document.addEventListener("mouseout", (event) => {
  if (floating.popup?.dataset.name !== "DropdownMenuContent") return;
  const menu = floating.popup.closest('[data-hover-menu="true"]');
  if (
    menu &&
    !menu.contains(
      event.relatedTarget instanceof Node ? event.relatedTarget : null,
    ) &&
    !floating.popup.contains(document.activeElement)
  ) {
    const leaving = floating.popup;
    setTimeout(() => {
      if (
        floating.popup === leaving &&
        !menu.matches(":hover") &&
        !leaving.contains(document.activeElement)
      )
        closePopup();
    }, 120);
  }
});
document.addEventListener("focusin", (event) => {
  const menu = closestControl(event.target, '[data-hover-menu="true"]');
  if (menu) {
    const content = controls(menu, '[data-name="DropdownMenuContent"]')[0];
    if (content) openPopup(content, triggerFor(content), false);
  }
});
