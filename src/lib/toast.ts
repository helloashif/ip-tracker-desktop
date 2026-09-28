import { reactive } from "vue";

export interface Toast { id: number; text: string; tone: "info" | "ok" | "error" }

export const toasts = reactive<Toast[]>([]);
let seq = 0;

export function toast(text: string, tone: Toast["tone"] = "info", ms = 3000) {
  const id = ++seq;
  toasts.push({ id, text, tone });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, ms);
}
