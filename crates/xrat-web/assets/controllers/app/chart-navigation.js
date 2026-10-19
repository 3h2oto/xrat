// @ts-check

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
});
