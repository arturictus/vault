// src/lib/stores/toaster.ts
import { tick } from 'svelte';

export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface ToastOptions {
  type?: ToastType;
  duration?: number;
}

export class Toast {
  readonly id: string = crypto.randomUUID();
  
  // State using new runes syntax
  message = $state('');
  type = $state<ToastType>('info');
  duration = $state(3000);
  timestamp = $state(Date.now());

  constructor(message: string, options: ToastOptions = {}) {
    this.message = message;
    this.type = options.type || 'info';
    this.duration = options.duration ?? 3000;
    this.timestamp = Date.now();
  }
}

export class ToasterStore {
  // State using new runes syntax
  toasts = $state<Toast[]>([]);

  addToast(message: string, options: ToastOptions = {}): string {
    const toast = new Toast(message, options);
    this.toasts = [...this.toasts, toast];
    
    if (options.duration !== 0) {
      setTimeout(() => {
        this.removeToast(toast.id);
      }, toast.duration);
    }
    
    return toast.id;
  }

  removeToast(id: string): void {
    this.toasts = this.toasts.filter(toast => toast.id !== id);
  }

  success(message: string, options: Omit<ToastOptions, 'type'> = {}): string {
    return this.addToast(message, { ...options, type: 'success' });
  }
  
  error(message: string, options: Omit<ToastOptions, 'type'> = {}): string {
    return this.addToast(message, { ...options, type: 'error' });
  }
  
  warning(message: string, options: Omit<ToastOptions, 'type'> = {}): string {
    return this.addToast(message, { ...options, type: 'warning' });
  }
  
  info(message: string, options: Omit<ToastOptions, 'type'> = {}): string {
    return this.addToast(message, { ...options, type: 'info' });
  }

  // Clear all toasts
  clear(): void {
    this.toasts = [];
  }
}

// Create and export a singleton instance
export const toaster = new ToasterStore();