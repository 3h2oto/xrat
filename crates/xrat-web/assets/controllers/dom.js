// @ts-check

/**
 * Find a component control from a delegated event target.
 * @param {EventTarget | null} target
 * @param {string} selector
 * @returns {HTMLElement | null}
 */
export function closestControl(target, selector) {
  if (!(target instanceof Element)) return null;
  const control = target.closest(selector);
  return control instanceof HTMLElement ? control : null;
}

/**
 * @param {ParentNode} root
 * @param {string} selector
 * @returns {HTMLElement[]}
 */
export function controls(root, selector) {
  return [...root.querySelectorAll(selector)].filter(
    (node) => node instanceof HTMLElement,
  );
}
