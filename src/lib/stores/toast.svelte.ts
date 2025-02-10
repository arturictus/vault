import { v4 as uuidv4 } from 'uuid';

interface Toast {
    id: string;
    message: string;
    type: 'success' | 'error' | 'info' | 'warning';
    duration: number;
}

let toasts = $state<Toast[]>([]);

export function addToast(
    message: string, 
    type: Toast['type'] = 'error', 
    duration: number = 5000
) {
    const id = uuidv4();
    const toast = { id, message, type, duration };
    toasts = [...toasts, toast];

    setTimeout(() => {
        toasts = toasts.filter(t => t.id !== id);
    }, duration);
}

export function getToasts() {
    return toasts;
}
