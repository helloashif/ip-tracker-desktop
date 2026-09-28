/** @type {import('tailwindcss').Config} */
const v = (name) => `rgb(var(${name}) / <alpha-value>)`;

export default {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  darkMode: ["selector", '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        bg: v("--c-bg"),
        surface: v("--c-surface"),
        surface2: v("--c-surface2"),
        line: v("--c-border"),
        fg: v("--c-text"),
        fg2: v("--c-text2"),
        fg3: v("--c-text3"),
        accent: v("--c-accent"),
        "accent-fg": v("--c-accent-fg"),
        warn: v("--c-warn"),
        danger: v("--c-danger"),
        ok: v("--c-ok"),
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "-apple-system", "Segoe UI", "Roboto", "sans-serif"],
      },
      boxShadow: {
        card: "0 1px 2px rgb(0 0 0 / 0.04), 0 1px 1px rgb(0 0 0 / 0.02)",
        pop: "0 12px 32px -8px rgb(0 0 0 / 0.35)",
      },
    },
  },
  plugins: [],
};
