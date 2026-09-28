import type { Theme } from "./api";

const mq = window.matchMedia("(prefers-color-scheme: dark)");
let current: Theme = "system";

function apply() {
  const resolved = current === "system" ? (mq.matches ? "dark" : "light") : current;
  document.documentElement.setAttribute("data-theme", resolved);
}

mq.addEventListener("change", apply);

export function setTheme(t: Theme) {
  current = t;
  apply();
}
