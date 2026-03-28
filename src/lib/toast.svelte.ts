export type ToastType = 'error' | 'success' | 'info';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  duration: number;
}

let nextId = 0;
let toasts: Toast[] = $state([]);

function add(message: string, type: ToastType, duration: number) {
  const id = nextId++;
  toasts.push({ id, message, type, duration });
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
}

function dismiss(id: number) {
  toasts = toasts.filter(t => t.id !== id);
}

export const toast = {
  get all(): Toast[] { return toasts; },
  error(message: string) { add(message, 'error', 0); },
  success(message: string) { add(message, 'success', 4000); },
  info(message: string) { add(message, 'info', 4000); },
  dismiss,
};
