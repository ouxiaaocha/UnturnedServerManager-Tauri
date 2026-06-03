export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  duration: number;
}

let nextId = 0;
let toastsList: Toast[] = $state([]);
let timers = new Map<number, ReturnType<typeof setTimeout>>();

export const toastStore = {
  get toasts() {
    return toastsList;
  },

  show(message: string, type: Toast["type"] = "info", duration = 3000) {
    const id = nextId++;
    const toast: Toast = { id, message, type, duration };
    toastsList = [...toastsList, toast];

    if (duration > 0) {
      const timer = setTimeout(() => {
        this.dismiss(id);
      }, duration);
      timers.set(id, timer);
    }

    return id;
  },

  success(message: string, duration = 3000) {
    return this.show(message, "success", duration);
  },

  error(message: string, duration = 4000) {
    return this.show(message, "error", duration);
  },

  info(message: string, duration = 3000) {
    return this.show(message, "info", duration);
  },

  dismiss(id: number) {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
    toastsList = toastsList.filter((t) => t.id !== id);
  },

  clear() {
    timers.forEach((timer) => clearTimeout(timer));
    timers.clear();
    toastsList = [];
  },
};
