import { writable } from 'svelte/store';

// Store that will trigger reload operations
export const refreshTrigger = writable(0);

// Call this function whenever you need to refresh data
export function triggerRefresh() {
  refreshTrigger.update(n => n + 1);
}
