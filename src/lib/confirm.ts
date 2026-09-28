import { reactive } from "vue";

interface ConfirmState {
  open: boolean;
  title: string;
  body: string;
  confirmLabel: string;
  danger: boolean;
  resolve: ((v: boolean) => void) | null;
}

export const confirmState = reactive<ConfirmState>({
  open: false, title: "", body: "", confirmLabel: "Confirm", danger: false, resolve: null,
});

export function confirm(opts: { title: string; body: string; confirmLabel?: string; danger?: boolean }): Promise<boolean> {
  return new Promise((resolve) => {
    Object.assign(confirmState, {
      open: true,
      title: opts.title,
      body: opts.body,
      confirmLabel: opts.confirmLabel ?? "Confirm",
      danger: opts.danger ?? false,
      resolve,
    });
  });
}

export function settle(v: boolean) {
  confirmState.resolve?.(v);
  confirmState.open = false;
  confirmState.resolve = null;
}
