// @ts-check
import { closestControl, controls } from "../dom.js";
import { floating, closeTooltip, showFloating } from "./floating.js";

/**
 * @param {HTMLElement} root
 */
function positionTooltip(root) {
  const content = controls(root, '[role="tooltip"]')[0];
  if (!content || floating.popup) return;
  if (floating.tooltip && floating.tooltip !== content) closeTooltip();
  floating.tooltip = content;
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
  const tooltipRoot = closestControl(event.target, '[data-name="Tooltip"]');
  if (tooltipRoot) positionTooltip(tooltipRoot);
});
document.addEventListener("mouseout", (event) => {
  const tipRoot = floating.tooltip?.closest('[data-name="Tooltip"]');
  if (
    tipRoot &&
    !tipRoot.contains(
      event.relatedTarget instanceof Node ? event.relatedTarget : null,
    ) &&
    !tipRoot.contains(document.activeElement)
  )
    closeTooltip();
});
document.addEventListener("focusin", (event) => {
  const tipRoot = floating.tooltip?.closest('[data-name="Tooltip"]');
  if (
    tipRoot &&
    !tipRoot.contains(event.target instanceof Node ? event.target : null)
  )
    closeTooltip();
  const tooltipRoot = closestControl(event.target, '[data-name="Tooltip"]');
  if (tooltipRoot) positionTooltip(tooltipRoot);
});
