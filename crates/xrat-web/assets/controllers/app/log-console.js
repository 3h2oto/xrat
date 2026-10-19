// @ts-check
import { closestControl } from "../dom.js";

document.addEventListener("click", (event) => {
  const button = closestControl(event.target, "[data-log-top]");
  if (button)
    document
      .getElementById(button.dataset.logTop ?? "")
      ?.scrollTo({ top: 0, left: 0, behavior: "auto" });
});
