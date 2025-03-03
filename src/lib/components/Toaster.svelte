<script lang="ts">
    import { toaster } from "$lib/stores/toaster.svelte";
    import { fade, fly } from "svelte/transition";

    type ToastPosition =
        | "top-right"
        | "top-left"
        | "bottom-right"
        | "bottom-left";

    // Props with default values using runes-style props
    let { 
        position = "top-right" as ToastPosition, 
        maxToasts = 5 
    } = $props();

    // Create a computed value for sorted toasts
    const sortedToasts = $derived(
        [...toaster.toasts]
            .sort((a, b) => b.timestamp - a.timestamp)
            .slice(0, maxToasts)
    );

    function getTransitionY(pos: ToastPosition): number {
        return pos.startsWith("top") ? -20 : 20;
    }
</script>

<div class="toaster {position}">
    {#each sortedToasts as toast (toast.id)}
        <div
            class="toast toast-{toast.type}"
            in:fly={{ y: getTransitionY(position), duration: 300 }}
            out:fade={{ duration: 200 }}
            on:click={() => toaster.removeToast(toast.id)}
        >
            <div class="toast-content">
                <span class="toast-message">{toast.message}</span>
                <button
                    class="toast-close"
                    aria-label="Close notification"
                    on:click|stopPropagation={() => toaster.removeToast(toast.id)}
                >
                    &times;
                </button>
            </div>
        </div>
    {/each}
</div>

<style>
    .toaster {
        position: fixed;
        display: flex;
        flex-direction: column;
        gap: 8px;
        z-index: 9999;
        pointer-events: none;
        padding: 16px;
        max-width: 100%;
    }

    .top-right {
        top: 0;
        right: 0;
        align-items: flex-end;
    }

    .top-left {
        top: 0;
        left: 0;
        align-items: flex-start;
    }

    .bottom-right {
        bottom: 0;
        right: 0;
        align-items: flex-end;
        flex-direction: column-reverse;
    }

    .bottom-left {
        bottom: 0;
        left: 0;
        align-items: flex-start;
        flex-direction: column-reverse;
    }

    .toast {
        padding: 12px 16px;
        border-radius: 4px;
        background-color: white;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
        pointer-events: auto;
        width: 300px;
        max-width: calc(100vw - 32px);
        cursor: pointer;
    }

    .toast-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .toast-message {
        flex: 1;
        word-break: break-word;
    }

    .toast-info {
        border-left: 4px solid #3498db;
    }

    .toast-success {
        border-left: 4px solid #2ecc71;
    }

    .toast-warning {
        border-left: 4px solid #f39c12;
    }

    .toast-error {
        border-left: 4px solid #e74c3c;
    }

    .toast-close {
        background: none;
        border: none;
        font-size: 18px;
        cursor: pointer;
        margin-left: 8px;
        color: #666;
        opacity: 0.7;
        transition: opacity 0.2s;
    }

    .toast-close:hover {
        opacity: 1;
    }
</style>
